mod udp;
mod mpsc_channel;
mod mpsc_corruptor;
pub use {
    udp::{UdpIp},
    mpsc_channel::{MpscChannel},
    mpsc_corruptor::{MpscCorruptor},
};
use {
    copernica_common::{
        InterLinkPacket, LinkId,
        constants,
        HBFI, ReplyTo,
        NarrowWaistPacket, ResponseData, LinkPacket, BFI,
        generate_nonce, Nonce, Tag, Data
    },
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    copernica_identity::{ PrivateIdentity, PublicIdentity, Signature },
    bincode,
    crossbeam_channel::{Sender, Receiver},
    anyhow::{anyhow, Result},
    reed_solomon::{Buffer, Encoder, Decoder},
};
fn u16_to_u8(i: u16) -> [u8; 2] {
    [(i >> 8) as u8, i as u8]
}
fn u8_to_u16(i: [u8; 2]) -> u16 {
    ((i[0] as u16) << 8) | i[1] as u16
}
fn bfi_to_u8(bfi: BFI) -> [u8; constants::BFI_BYTE_SIZE] {
    let mut bbfi: [u8; constants::BFI_BYTE_SIZE] = [0; constants::BFI_BYTE_SIZE];
    let mut count = 0;
    for i in bfi.iter() {
        let two_u8 = u16_to_u8(*i);
        bbfi[count]   = two_u8[0];
        bbfi[count+1] = two_u8[1];
        count+=2;
    }
    bbfi
}
fn u8_to_bfi(bbfi: [u8; constants::BFI_BYTE_SIZE]) -> BFI {
    [((bbfi[0] as u16) << 8) | bbfi[1] as u16,
    ((bbfi[2]  as u16) << 8) | bbfi[3] as u16,
    ((bbfi[4]  as u16) << 8) | bbfi[5] as u16,
    ((bbfi[6]  as u16) << 8) | bbfi[7] as u16]
}
fn u8_to_u64(v: [u8; 8]) -> u64 {
    let mut x: u64 = 0;
    x = ((x << 56) | v[0] as u64) as u64;
    x = ((x << 48) | v[1] as u64) as u64;
    x = ((x << 40) | v[2] as u64) as u64;
    x = ((x << 32) | v[3] as u64) as u64;
    x = ((x << 24) | v[4] as u64) as u64;
    x = ((x << 16) | v[5] as u64) as u64;
    x = ((x << 8)  | v[6] as u64) as u64;
    x = (x         | v[7] as u64) as u64;
    x
}
fn u64_to_u8(x: u64) -> [u8; 8] {
    [((x >> 56) & 0xff) as u8,
    ((x  >> 48) & 0xff) as u8,
    ((x  >> 40) & 0xff) as u8,
    ((x  >> 32) & 0xff) as u8,
    ((x  >> 24) & 0xff) as u8,
    ((x  >> 16) & 0xff) as u8,
    ((x  >> 8)  & 0xff) as u8,
    (x          & 0xff) as u8]
}
pub fn serialize_response_data(rd: &ResponseData) -> Vec<u8> {
    let mut buf: Vec<u8> = vec![];
    match rd {
        ResponseData::ClearText { data } => {
            buf.extend_from_slice(&data.raw_data());
            buf
        },
        ResponseData::CypherText { data, tag } => {
            buf.extend_from_slice(tag.as_ref());
            buf.extend_from_slice(&data.raw_data());
            buf
        },
    }
}
pub fn deserialize_cyphertext_response_data(data: &Vec<u8>) -> Result<ResponseData> {
    let mut tag = [0u8; constants::TAG_SIZE];
    tag.clone_from_slice(&data[..constants::TAG_SIZE]);
    let data = Data::new(data[constants::TAG_SIZE..].to_vec())?;
    Ok(ResponseData::reconstitute_cypher_text(tag, data))
}
pub fn deserialize_cleartext_response_data(data: &Vec<u8>) -> Result<ResponseData> {
    let data = Data::new(data[..].to_vec())?;
    Ok(ResponseData::reconstitute_clear_text(data))
}
fn serialize_hbfi(hbfi: &HBFI) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![];
    let res = &bfi_to_u8(hbfi.res);
    let req = &bfi_to_u8(hbfi.req);
    let app = &bfi_to_u8(hbfi.app);
    let m0d = &bfi_to_u8(hbfi.m0d);
    let fun = &bfi_to_u8(hbfi.fun);
    let arg = &bfi_to_u8(hbfi.arg);
    let ost = &u64_to_u8(hbfi.ost);
    let mut ids_buf: Vec<u8> = vec![];
    match &hbfi.request_pid {
        Some(request_pid) => {
            ids_buf.extend_from_slice(hbfi.response_pid.key().as_ref());
            ids_buf.extend_from_slice(hbfi.response_pid.chain_code().as_ref());
            ids_buf.extend_from_slice(request_pid.key().as_ref());
            ids_buf.extend_from_slice(request_pid.chain_code().as_ref());
        },
        None => {
            ids_buf.extend_from_slice(hbfi.response_pid.key().as_ref());
            ids_buf.extend_from_slice(hbfi.response_pid.chain_code().as_ref());
        },
    }
    buf.extend_from_slice(res);
    buf.extend_from_slice(req);
    buf.extend_from_slice(app);
    buf.extend_from_slice(m0d);
    buf.extend_from_slice(fun);
    buf.extend_from_slice(arg);
    buf.extend_from_slice(ost);
    buf.extend_from_slice(&ids_buf);
    Ok(buf)
}
pub fn deserialize_hbfi(data: &Vec<u8>) -> Result<HBFI> {
    let mut cnt = 0;
    let mut bfis: Vec<BFI> = Vec::with_capacity(constants::BFI_COUNT);
    for _ in 0..constants::BFI_COUNT {
        let mut bbfi: [u8; constants::BFI_BYTE_SIZE] = [0; constants::BFI_BYTE_SIZE];
        bbfi.clone_from_slice(&data[cnt..cnt+constants::BFI_BYTE_SIZE]);
        cnt += constants::BFI_BYTE_SIZE;
        bfis.push(u8_to_bfi(bbfi));
    }
    let mut ost: [u8; constants::U64_SIZE] = [0; constants::U64_SIZE];
    ost.clone_from_slice(&data[cnt..cnt+constants::U64_SIZE]);
    let ost: u64 = u8_to_u64(ost);
    cnt += constants::U64_SIZE;
    match data.len() {
        constants::CYPHERTEXT_HBFI_SIZE => {
            let mut res_key: [u8; constants::ID_SIZE] = [0; constants::ID_SIZE];
            res_key.clone_from_slice(&data[cnt..cnt+constants::ID_SIZE]);
            cnt += constants::ID_SIZE;
            let mut res_ccd: [u8; constants::CC_SIZE] = [0; constants::CC_SIZE];
            res_ccd.clone_from_slice(&data[cnt..cnt+constants::CC_SIZE]);
            cnt += constants::CC_SIZE;
            let mut req_key: [u8; constants::ID_SIZE] = [0; constants::ID_SIZE];
            req_key.clone_from_slice(&data[cnt..cnt+constants::ID_SIZE]);
            cnt += constants::ID_SIZE;
            let mut req_ccd: [u8; constants::CC_SIZE] = [0; constants::CC_SIZE];
            req_ccd.clone_from_slice(&data[cnt..cnt+constants::CC_SIZE]);
            Ok(HBFI { response_pid: PublicIdentity::reconstitute(res_key, res_ccd)
                    , request_pid: Some(PublicIdentity::reconstitute(req_key, req_ccd))
                    , res: bfis[0], req: bfis[1], app: bfis[2] , m0d: bfis[3], fun: bfis[4], arg: bfis[5]
                    , ost})
        },
        constants::CLEARTEXT_HBFI_SIZE => {
            let mut res_key: [u8; constants::ID_SIZE] = [0; constants::ID_SIZE];
            res_key.clone_from_slice(&data[cnt..cnt+constants::ID_SIZE]);
            cnt += constants::ID_SIZE;
            let mut res_ccd: [u8; constants::CC_SIZE] = [0; constants::CC_SIZE];
            res_ccd.clone_from_slice(&data[cnt..cnt+constants::CC_SIZE]);
            Ok(HBFI { response_pid: PublicIdentity::reconstitute(res_key, res_ccd)
                    , request_pid: None
                    , res: bfis[0], req: bfis[1], app: bfis[2] , m0d: bfis[3], fun: bfis[4], arg: bfis[5]
                    , ost})
        },
        _ => Err(anyhow!("The HBFI is of an unknown length")),
    }
}
pub fn deserialize_narrow_waist_packet_response(data: &Vec<u8>) -> Result<NarrowWaistPacket> {
    let mut cnt = 0;
    let hbfi: HBFI = deserialize_hbfi(&data[cnt..cnt+constants::CYPHERTEXT_HBFI_SIZE].to_vec())?;
    cnt += constants::CYPHERTEXT_HBFI_SIZE;
    let mut signature: [u8; Signature::SIZE] = [0; Signature::SIZE];
    signature.clone_from_slice(&data[cnt..cnt+Signature::SIZE]);
    cnt += Signature::SIZE;
    let signature: Signature = Signature::reconstitute(&signature);
    let mut offset: [u8; constants::U64_SIZE] = [0; constants::U64_SIZE];
    offset.clone_from_slice(&data[cnt..cnt+constants::U64_SIZE]);
    let offset: u64 = u8_to_u64(offset);
    cnt += constants::U64_SIZE;
    let mut total: [u8; constants::U64_SIZE] = [0; constants::U64_SIZE];
    total.clone_from_slice(&data[cnt..cnt+constants::U64_SIZE]);
    let total: u64 = u8_to_u64(total);
    cnt += constants::U64_SIZE;
    let mut nonce: [u8; constants::NONCE_SIZE] = [0; constants::NONCE_SIZE];
    nonce.clone_from_slice(&data[cnt..cnt+constants::NONCE_SIZE]);
    cnt += constants::NONCE_SIZE;
    let data: ResponseData = deserialize_cyphertext_response_data(&data[cnt..cnt+constants::CYPHERTEXT_RESPONSE_DATA_SIZE].to_vec())?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::Response { hbfi, signature, offset, total, nonce, data };
    Ok(nw)
}
pub fn deserialize_narrow_waist_packet_request(data: &Vec<u8>) -> Result<NarrowWaistPacket> {
    let mut cnt = 0;
    let hbfi: HBFI = deserialize_hbfi(&data[cnt..cnt+constants::CYPHERTEXT_HBFI_SIZE].to_vec())?;
    cnt += constants::CYPHERTEXT_HBFI_SIZE;
    let mut signature: [u8; Signature::SIZE] = [0; Signature::SIZE];
    signature.clone_from_slice(&data[cnt..cnt+Signature::SIZE]);
    cnt += Signature::SIZE;
    let signature: Signature = Signature::reconstitute(&signature);
    let mut offset: [u8; constants::U64_SIZE] = [0; constants::U64_SIZE];
    offset.clone_from_slice(&data[cnt..cnt+constants::U64_SIZE]);
    let offset: u64 = u8_to_u64(offset);
    cnt += constants::U64_SIZE;
    let mut total: [u8; constants::U64_SIZE] = [0; constants::U64_SIZE];
    total.clone_from_slice(&data[cnt..cnt+constants::U64_SIZE]);
    let total: u64 = u8_to_u64(total);
    cnt += constants::U64_SIZE;
    let mut nonce: [u8; constants::NONCE_SIZE] = [0; constants::NONCE_SIZE];
    nonce.clone_from_slice(&data[cnt..cnt+constants::NONCE_SIZE]);
    cnt += constants::NONCE_SIZE;
    let data: ResponseData = deserialize_cleartext_response_data(&data[cnt..cnt+constants::CLEARTEXT_RESPONSE_DATA_SIZE].to_vec())?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::Response { hbfi, signature, offset, total, nonce, data };
    Ok(nw)
}
pub fn serialize_narrow_waist_packet(nw: &NarrowWaistPacket) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![];
    match nw {
        NarrowWaistPacket::Request { hbfi, nonce } => {
            let hbfi = serialize_hbfi(&hbfi)?;
            let length: u16 = hbfi.len() as u16 + nonce.len() as u16;
            let length = u16_to_u8(length);
            buf.extend_from_slice(&length);
            buf.extend_from_slice(&hbfi);
            buf.extend_from_slice(nonce);
        },
        NarrowWaistPacket::Response { hbfi, signature, offset, total, nonce, data } => {
            let hbfi = serialize_hbfi(&hbfi)?;
            let response_data = serialize_response_data(&data);
            let ost = &u64_to_u8(*offset);
            let tot = &u64_to_u8(*total);
            let response_length: u16 = hbfi.len() as u16
                + signature.as_ref().len() as u16
                + ost.len() as u16
                + tot.len() as u16
                + nonce.len() as u16
                + response_data.len() as u16;
            let length = u16_to_u8(response_length);
            buf.extend_from_slice(&length);
            buf.extend_from_slice(&hbfi);
            buf.extend_from_slice(signature.as_ref());
            buf.extend_from_slice(ost);
            buf.extend_from_slice(tot);
            buf.extend_from_slice(nonce);
            buf.extend_from_slice(&response_data);
        },
    }
    Ok(buf)
}

fn serialize_reply_to(rt: &ReplyTo) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![];
    match rt {
        ReplyTo::Mpsc => {
            let length: u8 = 0;
            buf.extend_from_slice([length].as_ref());
        },
        ReplyTo::UdpIp(addr) => {
            let addr_s = bincode::serialize(&addr)?;
            let length: u8 = addr_s.len() as u8;
            buf.extend_from_slice([length].as_ref());
            buf.extend_from_slice(addr_s.as_ref());
        }
        ReplyTo::Rf(hz) => {
            let hz = bincode::serialize(&hz)?;
            let length: u8 = hz.len() as u8;
            buf.extend_from_slice([length].as_ref());
            buf.extend_from_slice(hz.as_ref());
        }
    }
    Ok(buf)
}

fn deserialize_reply_to(data: &Vec<u8>) -> Result<ReplyTo> {
    let length = data.len();
    let rt = match length as usize {
        constants::TO_REPLY_TO_MPSC => {
            ReplyTo::Mpsc
        },
        constants::TO_REPLY_TO_UDPIP4 => {
            let address = &data[..constants::TO_REPLY_TO_UDPIP4];
            let address = bincode::deserialize(&address)?;
            ReplyTo::UdpIp(address)
        },
        constants::TO_REPLY_TO_UDPIP6 => {
            let address = &data[..constants::TO_REPLY_TO_UDPIP6];
            let address = bincode::deserialize(&address)?;
            ReplyTo::UdpIp(address)
        },
        constants::TO_REPLY_TO_RF => {
            let address = &data[..constants::TO_REPLY_TO_RF];
            let address = bincode::deserialize(&address)?;
            ReplyTo::Rf(address)
        },
        _ => return Err(anyhow!("Deserializing ReplyTo hit an unrecognised type or variation"))
    };
    Ok(rt)
}

const ONE_BYTE: usize = 1;
const TWO_BYTE: usize = 2;

pub fn serialize_link_packet(lp: &LinkPacket, lnk_tx_sid: PrivateIdentity, lnk_rx_pid: Option<PublicIdentity>) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![];
    match lnk_rx_pid {
        None => {
            let reply_to = lp.reply_to();
            let nw = lp.narrow_waist();
            let byte: u8 = 0;
            buf.extend_from_slice([byte].as_ref());
            buf.extend_from_slice(lnk_tx_sid.public_id().key().as_ref());
            buf.extend_from_slice(lnk_tx_sid.public_id().chain_code().as_ref());
            let reply_to = serialize_reply_to(&reply_to)?;
            let nw = serialize_narrow_waist_packet(&nw)?;
            buf.extend_from_slice(&reply_to);
            buf.extend_from_slice(&nw);
        },
        Some(lnk_rx_pid) => {
            let reply_to = lp.reply_to();
            let nw = lp.narrow_waist();
            let byte: u8 = 1;
            buf.extend_from_slice([byte].as_ref());
    // Link Pid
            buf.extend_from_slice(lnk_tx_sid.public_id().key().as_ref());
    // Link CC
            buf.extend_from_slice(lnk_tx_sid.public_id().chain_code().as_ref());
    // Reply To
            let reply_to = serialize_reply_to(&reply_to)?;
            buf.extend_from_slice(&reply_to);
            let mut rng = rand::thread_rng();
            let nonce: Nonce = generate_nonce(&mut rng);

            let lnk_rx_pk = lnk_rx_pid.derive(&nonce);
            let lnk_tx_sk = lnk_tx_sid.derive(&nonce);
            let shared_secret = lnk_tx_sk.exchange(&lnk_rx_pk);
            let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce, &[]);
            let mut nws = serialize_narrow_waist_packet(&nw)?;
            let mut encrypted = vec![0u8; nws.len()];
            let mut tag: Tag = [0; constants::TAG_SIZE];
            ctx.encrypt(&nws, &mut encrypted[..], &mut tag);
    // Nonce
            buf.extend_from_slice(nonce.as_ref());
    // Tag
            buf.extend_from_slice(tag.as_ref());
            nws.copy_from_slice(&encrypted[..]);
            if constants::NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE != nws.len() {
                return Err(anyhow!("Sending packet has unrecognised NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE of {}, where it should be {}", nws.len(), constants::NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE));
            }
    // Narrow Waist
            buf.extend_from_slice(&nws);
        },
    }
    Ok(buf)
}

pub fn deserialize_link_packet(data: &Vec<u8>, lnk_rx_sid: PrivateIdentity) -> Result<LinkPacket> {
    let mut cnt = 0;
    let link_state = &data[0..ONE_BYTE];
    cnt += ONE_BYTE;
    let lp: LinkPacket = match link_state[0] as usize {
        0 => { todo!() },
        1 => { // encrypted link_packet
    // Link Pid
            let mut link_tx_pk: [u8; constants::ID_SIZE] = [0; constants::ID_SIZE];
            link_tx_pk.clone_from_slice(&data[cnt..cnt+constants::ID_SIZE]);
            cnt += constants::ID_SIZE;
    // Link CC
            let mut link_tx_cc: [u8; constants::CC_SIZE] = [0; constants::CC_SIZE];
            link_tx_cc.clone_from_slice(&data[cnt..cnt+constants::CC_SIZE]);
            cnt += constants::CC_SIZE;
            let lnk_tx_pid: PublicIdentity = PublicIdentity::reconstitute(link_tx_pk, link_tx_cc);
    // Reply To
            let reply_to_length = &data[cnt..cnt+ONE_BYTE];
            cnt += ONE_BYTE;
            let reply_to: ReplyTo = deserialize_reply_to(&data[cnt..cnt + reply_to_length[0] as usize].to_vec())?;
            cnt += reply_to_length[0] as usize;
    // Nonce
            let mut link_nonce: [u8; constants::NONCE_SIZE] = [0; constants::NONCE_SIZE];
            link_nonce.clone_from_slice(&data[cnt..cnt+constants::NONCE_SIZE]);
            cnt += constants::NONCE_SIZE;
    // Tag
            let mut link_tag: [u8; constants::TAG_SIZE] = [0; constants::TAG_SIZE];
            link_tag.clone_from_slice(&data[cnt..cnt+constants::TAG_SIZE]);
            cnt += constants::TAG_SIZE;

            let lnk_tx_pk = lnk_tx_pid.derive(&link_nonce);
            let lnk_rx_sk = lnk_rx_sid.derive(&link_nonce);
            let shared_secret = lnk_rx_sk.exchange(&lnk_tx_pk);

            let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &link_nonce, &[]);
            let mut decrypted = vec![0u8; constants::NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE];
            let encrypted = &data[cnt..cnt+constants::NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE];
            if !ctx.decrypt(encrypted, &mut decrypted, &link_tag) {
                return Err(anyhow!("failed to decrypt link packet"));
            };
            let mut cnt = 0;
            let mut length: [u8; 2] = [0; 2];
            length.clone_from_slice(&decrypted[cnt..cnt+TWO_BYTE]);
            let length: u16 = u8_to_u16(length);
            //println!("{}", length);
            cnt += TWO_BYTE;
    // Narrow Waist
            let nw: NarrowWaistPacket = match length as usize{
                45 => { deserialize_narrow_waist_packet_request(&decrypted[cnt..cnt+length as usize].to_vec())?},
                //constants::NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE
                _ => {
                    deserialize_narrow_waist_packet_response(&decrypted[cnt..cnt+length as usize].to_vec())?
                },
                //_ => {
                //    return Err(anyhow!("Packet arrived with an unrecognised NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE of {}, where it should be {}",length, constants::NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE));
                //},
            };
            LinkPacket::new(reply_to, nw)
        }
        _ => return Err(anyhow!("Only two variants: 0 for cleartext and 1 for cyphertext LinkPackets, anything else is incorrect"))
    };
    Ok(lp)
}
pub fn decode(msg: Vec<u8>, lnk_rx_sid: PrivateIdentity) -> Result<LinkPacket> {
    let dec = Decoder::new(6);
    let reconstituted: Vec<_> = msg.chunks(255).map(|c| Buffer::from_slice(c, c.len())).map(|d| dec.correct(&d,None).unwrap()).collect();
    let reconstituted: Vec<_> = reconstituted.iter().map(|d| d.data()).collect::<Vec<_>>().concat();
    let lp: LinkPacket = deserialize_link_packet(&reconstituted, lnk_rx_sid)?;
    Ok(lp)
}
pub fn encode(lp: LinkPacket, lnk_tx_sid: PrivateIdentity, lnk_rx_pid: Option<PublicIdentity>) -> Result<Vec<u8>> {
    let mut merged = vec![];
    let enc = Encoder::new(6);
    let nw: Vec<u8> = serialize_link_packet(&lp, lnk_tx_sid, lnk_rx_pid)?;
    let cs = nw.chunks(255-6);
    for c in cs {
        let c = enc.encode(&c[..]);
        merged.extend(&**c);
    }
    Ok(merged)
}
pub trait Link<'a> {
    fn run(&self) -> Result<()>;
    fn new(name: String, link: LinkId, router_in_and_out: ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> ) ) -> Result<Self> where Self: Sized;
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_u16_to_fro_u8() {
        let actual: u16 = u16::MIN;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("{:?}, {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u16 = 1;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("{:?}, {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u16 = u16::MAX;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("{:?}, {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_bfi_to_fro_u8() {
        let actual: BFI = [0u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("{:?}, {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: BFI = [0, 1, 2, 3];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("{:?}, {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: BFI = [u16::MAX, u16::MAX, u16::MAX, u16::MAX];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("{:?}, {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
}
