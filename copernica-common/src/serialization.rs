use {
    crate::{
        constants::*,
        common::*,
        HBFI, ReplyTo, LinkId,
        NarrowWaistPacket, ResponseData, LinkPacket, BFI,
        PublicIdentity, Signature
    },
    macaddr::{MacAddr6, MacAddr8},
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    log::{trace, error},
    anyhow::{anyhow, Result},
};
fn u16_to_u8(i: u16) -> [u8; 2] {
    [(i >> 8) as u8, i as u8]
}
fn u8_to_u16(i: [u8; 2]) -> u16 {
    ((i[0] as u16) << 8) | i[1] as u16
}
fn bfi_to_u8(bfi: BFI) -> [u8; BFI_BYTE_SIZE] {
    let mut bbfi: [u8; BFI_BYTE_SIZE] = [0; BFI_BYTE_SIZE];
    let mut count = 0;
    for i in bfi.iter() {
        let two_u8 = u16_to_u8(*i);
        bbfi[count]   = two_u8[0];
        bbfi[count+1] = two_u8[1];
        count+=2;
    }
    bbfi
}
fn u8_to_bfi(bbfi: [u8; BFI_BYTE_SIZE]) -> BFI {
    [((bbfi[0] as u16) << 8) | bbfi[1] as u16,
    ((bbfi[2]  as u16) << 8) | bbfi[3] as u16,
    ((bbfi[4]  as u16) << 8) | bbfi[5] as u16,
    ((bbfi[6]  as u16) << 8) | bbfi[7] as u16]
}
pub fn u8_to_u64(v: [u8; 8]) -> u64 {
    let mut x: u64 = 0;
    for i in 0..v.len() {
        x = ((x << 8) | v[i] as u64) as u64;
    }
    x
}
pub fn u64_to_u8(x: u64) -> [u8; 8] {
    [((x >> 56) & 0xff) as u8,
    ((x  >> 48) & 0xff) as u8,
    ((x  >> 40) & 0xff) as u8,
    ((x  >> 32) & 0xff) as u8,
    ((x  >> 24) & 0xff) as u8,
    ((x  >> 16) & 0xff) as u8,
    ((x  >> 8)  & 0xff) as u8,
    (x          & 0xff) as u8]
}
pub fn serialize_response_data(rd: &ResponseData) -> (u16, Vec<u8>) {
    let mut buf: Vec<u8> = vec![];
    match rd {
        ResponseData::ClearText { data } => {
            buf.extend_from_slice(&data.raw_data());
            (buf.len() as u16, buf)
        },
        ResponseData::CypherText { data, tag } => {
            buf.extend_from_slice(tag.as_ref());
            buf.extend_from_slice(&data.raw_data());
            (buf.len() as u16, buf)
        },
    }
}
pub fn deserialize_cyphertext_response_data(data: &Vec<u8>) -> Result<ResponseData> {
    let mut tag = [0u8; TAG_SIZE];
    tag.clone_from_slice(&data[..TAG_SIZE]);
    let data = Data::new(data[TAG_SIZE..].to_vec())?;
    Ok(ResponseData::reconstitute_cypher_text(tag, data))
}
pub fn deserialize_cleartext_response_data(data: &Vec<u8>) -> Result<ResponseData> {
    let data = Data::new(data[..].to_vec())?;
    Ok(ResponseData::reconstitute_clear_text(data))
}
pub fn serialize_hbfi(hbfi: &HBFI) -> Result<(u8, Vec<u8>)> {
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
    trace!("ser \thbfi 0: \t\t{:?}", res.as_ref());
    buf.extend_from_slice(req);
    trace!("ser \thbfi 1: \t\t{:?}", req.as_ref());
    buf.extend_from_slice(app);
    trace!("ser \thbfi 2: \t\t{:?}", app.as_ref());
    buf.extend_from_slice(m0d);
    trace!("ser \thbfi 3: \t\t{:?}", m0d.as_ref());
    buf.extend_from_slice(fun);
    trace!("ser \thbfi 4: \t\t{:?}", fun.as_ref());
    buf.extend_from_slice(arg);
    trace!("ser \thbfi 5: \t\t{:?}", arg.as_ref());
    buf.extend_from_slice(ost);
    trace!("ser \toffset: \t\t{:?}", ost.as_ref());
    buf.extend_from_slice(&ids_buf);
    trace!("ser \tids: \t\t\t{:?}", ids_buf);
    let size = res.len() + req.len() + app.len() + m0d.len() + fun.len() + arg.len() + ost.len() + ids_buf.len();
    Ok((size as u8, buf))
}
pub fn deserialize_cyphertext_hbfi(data: &Vec<u8>) -> Result<HBFI> {
    let mut bfis: Vec<BFI> = Vec::with_capacity(BFI_COUNT);
    let mut count = 0;
    for _ in 0..BFI_COUNT {
        let mut bbfi = [0u8; BFI_BYTE_SIZE];
        bbfi.clone_from_slice(&data[count..count+BFI_BYTE_SIZE]);
        trace!("des \thbfi {}: \t\t{:?}", count, bbfi.as_ref());
        bfis.push(u8_to_bfi(bbfi));
        count += BFI_BYTE_SIZE;
    }
    let mut ost = [0u8; U64_SIZE];
    ost.clone_from_slice(&data[HBFI_OFFSET_START..HBFI_OFFSET_END]);
    trace!("des \toffset: \t\t{:?}", ost.as_ref());
    let ost: u64 = u8_to_u64(ost);
    let mut res_key = [0u8; ID_SIZE + CC_SIZE];
    res_key.clone_from_slice(&data[HBFI_RESPONSE_KEY_START..HBFI_RESPONSE_KEY_END]);
    //trace!("des \tres_key: \t\t{:?}", res_key);
    let mut req_key = [0u8; ID_SIZE + CC_SIZE];
    req_key.clone_from_slice(&data[HBFI_REQUEST_KEY_START..HBFI_REQUEST_KEY_END]);
    //trace!("des \treq_key: \t\t{:?}", req_key);
    Ok(HBFI { response_pid: PublicIdentity::from(res_key)
            , request_pid: Some(PublicIdentity::from(req_key))
            , res: bfis[0], req: bfis[1], app: bfis[2], m0d: bfis[3], fun: bfis[4], arg: bfis[5]
            , ost})
}
pub fn deserialize_cleartext_hbfi(data: &Vec<u8>) -> Result<HBFI> {
    let mut bfis: Vec<BFI> = Vec::with_capacity(BFI_COUNT);
    let mut count = 0;
    for _ in 0..BFI_COUNT {
        let mut bbfi = [0u8; BFI_BYTE_SIZE];
        bbfi.clone_from_slice(&data[count..count+BFI_BYTE_SIZE]);
        trace!("des \thbfi {}: \t\t{:?}", count, bbfi.as_ref());
        bfis.push(u8_to_bfi(bbfi));
        count += BFI_BYTE_SIZE;
    }
    let mut ost = [0u8; U64_SIZE];
    ost.clone_from_slice(&data[HBFI_OFFSET_START..HBFI_OFFSET_END]);
    trace!("des \toffset: \t\t{:?}", ost.as_ref());
    let ost: u64 = u8_to_u64(ost);
    let mut res_key = [0u8; ID_SIZE + CC_SIZE];
    res_key.clone_from_slice(&data[HBFI_RESPONSE_KEY_START..HBFI_RESPONSE_KEY_END]);
    //trace!("des \tres_key: \t\t{:?}", res_key);
    Ok(HBFI { response_pid: PublicIdentity::from(res_key)
            , request_pid: None
            , res: bfis[0], req: bfis[1], app: bfis[2], m0d: bfis[3], fun: bfis[4], arg: bfis[5]
            , ost})
}
pub fn deserialize_cyphertext_narrow_waist_packet_response(data: &Vec<u8>) -> Result<NarrowWaistPacket> {
    let mut signature = [0u8; Signature::SIZE];
    signature.clone_from_slice(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
    trace!("des \tsignature: \t\t{:?}", signature.as_ref());
    let signature: Signature = Signature::from(signature);
    let mut offset = [0u8; U64_SIZE];
    offset.clone_from_slice(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_OFFSET_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_OFFSET_END]);
    trace!("des \toffset: \t\t{:?}", offset.as_ref());
    let offset: u64 = u8_to_u64(offset);
    let mut total = [0u8; U64_SIZE];
    total.clone_from_slice(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_TOTAL_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_TOTAL_END]);
    trace!("des \ttotal: \t\t\t{:?}", total.as_ref());
    let total: u64 = u8_to_u64(total);
    let mut nonce = [0u8; NONCE_SIZE];
    nonce.clone_from_slice(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
    trace!("des \tnonce: \t\t\t{:?}", nonce.as_ref());
    let hbfi_end = CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END + CYPHERTEXT_HBFI_SIZE;
    let response_data_end = hbfi_end + CYPHERTEXT_RESPONSE_DATA_SIZE;
    let hbfi: HBFI = deserialize_cyphertext_hbfi(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END..hbfi_end].to_vec())?;
    let data: ResponseData = deserialize_cyphertext_response_data(&data[hbfi_end..response_data_end].to_vec())?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::Response { hbfi, signature, offset, total, nonce, data };
    Ok(nw)
}
pub fn deserialize_cleartext_narrow_waist_packet_response(data: &Vec<u8>) -> Result<NarrowWaistPacket> {
    let mut signature = [0u8; Signature::SIZE];
    signature.clone_from_slice(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
    trace!("des \tsignature: \t\t{:?}", signature.as_ref());
    let signature: Signature = Signature::from(signature);
    let mut offset = [0u8; U64_SIZE];
    offset.clone_from_slice(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_OFFSET_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_OFFSET_END]);
    trace!("des \toffset: \t\t{:?}", offset.as_ref());
    let offset: u64 = u8_to_u64(offset);
    let mut total = [0u8; U64_SIZE];
    total.clone_from_slice(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_TOTAL_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_TOTAL_END]);
    trace!("des \ttotal: \t\t\t{:?}", total.as_ref());
    let total: u64 = u8_to_u64(total);
    let mut nonce = [0u8; NONCE_SIZE];
    nonce.clone_from_slice(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
    trace!("des \tnonce: \t\t\t{:?}", nonce.as_ref());
    let hbfi_end = CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END + CLEARTEXT_HBFI_SIZE;
    let response_data_end = hbfi_end + CLEARTEXT_RESPONSE_DATA_SIZE;
    let hbfi: HBFI = deserialize_cleartext_hbfi(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END..hbfi_end].to_vec())?;
    let data: ResponseData = deserialize_cleartext_response_data(&data[hbfi_end..response_data_end].to_vec())?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::Response { hbfi, signature, offset, total, nonce, data };
    Ok(nw)
}
pub fn deserialize_cyphertext_narrow_waist_packet_request(data: &Vec<u8>) -> Result<NarrowWaistPacket> {
    let mut nonce = [0u8; NONCE_SIZE];
    nonce.clone_from_slice(&data[0..NONCE_SIZE]);
    let hbfi: HBFI = deserialize_cyphertext_hbfi(&data[NONCE_SIZE..NONCE_SIZE+CYPHERTEXT_HBFI_SIZE].to_vec())?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::Request { hbfi, nonce };
    Ok(nw)
}
pub fn deserialize_cleartext_narrow_waist_packet_request(data: &Vec<u8>) -> Result<NarrowWaistPacket> {
    let mut nonce = [0u8; NONCE_SIZE];
    nonce.clone_from_slice(&data[0..NONCE_SIZE]);
    let hbfi: HBFI = deserialize_cleartext_hbfi(&data[NONCE_SIZE..NONCE_SIZE+CLEARTEXT_HBFI_SIZE].to_vec())?;
    let nw: NarrowWaistPacket = NarrowWaistPacket::Request { hbfi, nonce };
    Ok(nw)
}
pub fn deserialize_narrow_waist_packet(data: &Vec<u8>) -> Result<NarrowWaistPacket> {
    match data.len() {
        CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
            deserialize_cyphertext_narrow_waist_packet_response(data)
        },
        CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
            deserialize_cyphertext_narrow_waist_packet_request(data)
        },
        CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
            deserialize_cleartext_narrow_waist_packet_response(data)
        },
        CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
            deserialize_cleartext_narrow_waist_packet_request(data)
        },
        _ => { return Err(anyhow!("Cannot deserialize this as the size is unknown")) }
    }
}
pub fn serialize_narrow_waist_packet(nw: &NarrowWaistPacket) -> Result<(u16, Vec<u8>)> {
    let mut buf: Vec<u8> = vec![];
    let size: u16;
    match nw {
        NarrowWaistPacket::Request { hbfi, nonce } => {
            let (hbfi_size, hbfi) = serialize_hbfi(&hbfi)?;
            size = hbfi_size as u16 + nonce.len() as u16;
            buf.extend_from_slice(nonce);
            buf.extend_from_slice(&hbfi);
        },
        NarrowWaistPacket::Response { hbfi, signature, offset, total, nonce, data } => {
            let (hbfi_size, hbfi) = serialize_hbfi(&hbfi)?;
            let (response_data_size, response_data) = serialize_response_data(&data);
            let ost = &u64_to_u8(*offset);
            let tot = &u64_to_u8(*total);
            size = hbfi_size as u16
                + signature.as_ref().len() as u16
                + ost.len() as u16
                + tot.len() as u16
                + nonce.len() as u16
                + response_data_size as u16;
            trace!("ser \tsignature: \t\t{:?}", signature.as_ref());
            buf.extend_from_slice(signature.as_ref());
            trace!("ser \toffset: \t\t{:?}", ost.as_ref());
            buf.extend_from_slice(ost);
            trace!("ser \ttotal: \t\t\t{:?}", tot.as_ref());
            buf.extend_from_slice(tot);
            trace!("ser \tnonce: \t\t\t{:?}", nonce.as_ref());
            buf.extend_from_slice(nonce);
            buf.extend_from_slice(&hbfi);
            buf.extend_from_slice(&response_data);
        },
    }
    Ok((size, buf))
}

fn serialize_reply_to(rt: &ReplyTo) -> Result<(u8, Vec<u8>)> {
    let mut buf: Vec<u8> = vec![];
    let size: u8;
    match rt {
        ReplyTo::Mpsc => {
            size = 0;
            trace!("ser rep_to mpsc: \t\t{:?}", [0]);
        },
        ReplyTo::UdpIp(addr) => {
            let addr_s = bincode::serialize(&addr)?;
            size = addr_s.len() as u8;
            trace!("ser rep_to udpip: \t\t{:?}", addr_s);
            buf.extend_from_slice(addr_s.as_ref());
        }
        ReplyTo::MacAddr6(addr) => {
            size = addr.as_bytes().len() as u8;
            trace!("ser rep_to macaddr6: \t\t{:?}", addr);
            buf.extend_from_slice(addr.as_ref());
        }
        ReplyTo::MacAddr8(addr) => {
            size = addr.as_bytes().len() as u8;
            trace!("ser rep_to macaddr8: \t\t{:?}", addr);
            buf.extend_from_slice(addr.as_ref());
        }
        ReplyTo::Rf(hz) => {
            let hz = bincode::serialize(&hz)?;
            size = hz.len() as u8;
            trace!("ser rep_to udpip: \t\t{:?}", hz);
            buf.extend_from_slice(hz.as_ref());
        }
    }
    Ok((size, buf))
}

fn deserialize_reply_to(data: &Vec<u8>) -> Result<ReplyTo> {
    let rt = match data.len() as usize {
        TO_REPLY_TO_MPSC => {
            ReplyTo::Mpsc
        },
        TO_REPLY_TO_UDPIP4 => {
            let address = &data[..];
            let address = bincode::deserialize(&address)?;
            ReplyTo::UdpIp(address)
        },
        TO_REPLY_TO_UDPIP6 => {
            let address = &data[..];
            let address = bincode::deserialize(&address)?;
            ReplyTo::UdpIp(address)
        },
        TO_REPLY_TO_MACADDR6 => {
            let mut address = [0u8; 6];
            address.copy_from_slice(&data[..]);
            let address = MacAddr6::from(address);
            ReplyTo::MacAddr6(address)
        },
        TO_REPLY_TO_MACADDR8 => {
            let mut address = [0u8; 8];
            address.copy_from_slice(&data[..]);
            let address = MacAddr8::from(address);
            ReplyTo::MacAddr8(address)
        },
        TO_REPLY_TO_RF => {
            let address = &data[..];
            let address = bincode::deserialize(&address)?;
            ReplyTo::Rf(address)
        },
        _ => return Err(anyhow!("Deserializing ReplyTo hit an unrecognised type or variation"))
    };
    Ok(rt)
}

pub fn serialize_link_packet(lp: &LinkPacket, link_id: LinkId) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = vec![];
    let lnk_tx_pid = link_id.tx_pid()?;
    match link_id.rx_pid()? {
        None => {
            let reply_to = lp.reply_to();
            let nw = lp.narrow_waist();
            buf.extend_from_slice(lnk_tx_pid.key().as_ref());
            trace!("ser link_key: \t\t\t{:?}", lnk_tx_pid.key().as_ref());
            buf.extend_from_slice(lnk_tx_pid.chain_code().as_ref());
            trace!("ser link_ccd: \t\t\t{:?}", lnk_tx_pid.chain_code().as_ref());
            let (reply_to_size, reply_to) = serialize_reply_to(&reply_to)?;
            trace!("ser reply_to_size: \t\t{:?}", reply_to_size);
            let (nw_size, nw) = serialize_narrow_waist_packet(&nw)?;
            trace!("ser nw_size: \t\t\t{:?}", nw_size);
            buf.extend_from_slice(&[reply_to_size]);
            buf.extend_from_slice(&u16_to_u8(nw_size));
            trace!("ser reply_to: \t\t\t{:?}", reply_to);
            buf.extend_from_slice(&reply_to);
            buf.extend_from_slice(&nw);
        },
        Some(lnk_rx_pid) => {
            let reply_to = lp.reply_to();
            let nw = lp.narrow_waist();
    // Link Pid
            buf.extend_from_slice(lnk_tx_pid.key().as_ref());
            trace!("ser link_tx_pk: \t\t{:?}", lnk_tx_pid.key().as_ref());
    // Link CC
            buf.extend_from_slice(lnk_tx_pid.chain_code().as_ref());
            trace!("ser link_cc_pk: \t\t{:?}", lnk_tx_pid.chain_code().as_ref());
    // Nonce
            let mut rng = rand::thread_rng();
            let nonce: Nonce = generate_nonce(&mut rng);
            buf.extend_from_slice(nonce.as_ref());
            trace!("ser link_nonce: \t\t{:?}", nonce.as_ref());
    // Tag
            let mut tag: Tag = [0; TAG_SIZE];
            let shared_secret = link_id.shared_secret(nonce, lnk_rx_pid)?;
            let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &nonce, &[]);
            drop(shared_secret);
            let (nws_size, mut nws) = serialize_narrow_waist_packet(&nw)?;
            let mut encrypted = vec![0u8; nws.len()];
            ctx.encrypt(&nws, &mut encrypted[..], &mut tag);
            nws.copy_from_slice(&encrypted[..]);
            buf.extend_from_slice(tag.as_ref());
            trace!("ser link_tag: \t\t\t{:?}", tag.as_ref());
    // Reply To Size
            let (reply_to_size, reply_to) = serialize_reply_to(&reply_to)?;
            buf.extend_from_slice(&[reply_to_size]);
            trace!("ser link_reply_to_size: \t{:?} actual_size: {}", [reply_to_size], reply_to.len());
    // Narrow Waist Size
            buf.extend_from_slice(&u16_to_u8(nws_size));
            trace!("ser nw_size: \t\t\t{:?} as_u16: {} actual {}", u16_to_u8(nws_size), nws_size, nws.len());
            buf.extend_from_slice(&reply_to);

    // Narrow Waist
            buf.extend_from_slice(&nws);
        },
    }
    Ok(buf)
}

pub fn deserialize_cyphertext_link_packet(data: &Vec<u8>, link_id: LinkId) -> Result<(PublicIdentity, LinkPacket)> {
// Link Pid
    let mut link_tx_pk_with_cc = [0u8; ID_SIZE + CC_SIZE];
    link_tx_pk_with_cc.clone_from_slice(&data[CYPHERTEXT_LINK_TX_PK_START..CYPHERTEXT_LINK_TX_PK_END]);
    //trace!("des link_tx_pk: \t\t{:?}", link_tx_pk);
    let lnk_tx_pid: PublicIdentity = PublicIdentity::from(link_tx_pk_with_cc);
// Nonce
    let mut link_nonce = [0u8; NONCE_SIZE];
    link_nonce.clone_from_slice(&data[CYPHERTEXT_LINK_NONCE_START..CYPHERTEXT_LINK_NONCE_END]);
    trace!("des link_nonce: \t\t{:?}", link_nonce);
// Tag
    let mut link_tag = [0u8; TAG_SIZE];
    link_tag.clone_from_slice(&data[CYPHERTEXT_LINK_TAG_START..CYPHERTEXT_LINK_TAG_END]);
    trace!("des link_tag: \t\t\t{:?}", link_tag);
// Reply To Length
    let reply_to_size = &data[CYPHERTEXT_LINK_REPLY_TO_SIZE_START..CYPHERTEXT_LINK_REPLY_TO_SIZE_END];
    trace!("des reply_to_size: \t\t{:?}", reply_to_size);
// Narrow Waist Length
    let mut nw_size = [0u8; 2];
    nw_size.clone_from_slice(&data[CYPHERTEXT_LINK_NARROW_WAIST_SIZE_START..CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END]);
    trace!("des nw_size: \t\t\t{:?} as_u16: {}", nw_size, u8_to_u16(nw_size));
    let nw_size: usize = u8_to_u16(nw_size) as usize;
    let reply_to: ReplyTo = deserialize_reply_to(&data[CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END..CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize].to_vec())?;
    trace!("des reply_to: \t\t\t{:?}", reply_to);
    let nw_start = CYPHERTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize;
    trace!("des nw_start: \t\t\t{:?}", nw_start);
    let shared_secret = link_id.shared_secret(link_nonce, lnk_tx_pid.clone())?;
    let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &link_nonce, &[]);
    drop(shared_secret);
    let nw: NarrowWaistPacket = match nw_size {
        CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
            let mut decrypted = vec![0u8; nw_size];
            let encrypted = &data[nw_start..nw_start + nw_size];
            //trace!("des encrypted: actual_length: {} NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE {}\t\t\t{:?} ", encrypted.len(), CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, encrypted);
            if !ctx.decrypt(encrypted, &mut decrypted, &link_tag) {
                let err_msg = "failed to decrypt link packet";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            };
            deserialize_cyphertext_narrow_waist_packet_request(&decrypted.to_vec())?
        },
        CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
            let mut decrypted = vec![0u8; nw_size];
            let encrypted = &data[nw_start..nw_start + nw_size];
            //trace!("des encrypted: actual_length: {} NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE {}\t\t\t{:?} ", encrypted.len(), CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, encrypted);
            if !ctx.decrypt(encrypted, &mut decrypted, &link_tag) {
                let err_msg = "failed to decrypt link packet";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            };
            deserialize_cyphertext_narrow_waist_packet_response(&decrypted.to_vec())?
        },
        CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
            let mut decrypted = vec![0u8; nw_size];
            let encrypted = &data[nw_start..nw_start + nw_size];
            //trace!("des encrypted: actual_length: {} NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE {}\t\t\t{:?} ", encrypted.len(), CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, encrypted);
            if !ctx.decrypt(encrypted, &mut decrypted, &link_tag) {
                let err_msg = "failed to decrypt link packet";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            };
            deserialize_cleartext_narrow_waist_packet_request(&decrypted.to_vec())?
        },
        CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
            let mut decrypted = vec![0u8; nw_size];
            let encrypted = &data[nw_start..nw_start + nw_size];
            //trace!("des encrypted: actual_length: {} NARROW_WAIST_PACKET_ENCRYPTED_RESPONSE_SIZE {}\t\t\t{:?} ", encrypted.len(), CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, encrypted);
            if !ctx.decrypt(encrypted, &mut decrypted, &link_tag) {
                let err_msg = "failed to decrypt link packet";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            };
            deserialize_cleartext_narrow_waist_packet_response(&decrypted.to_vec())?
        },
        _ => {
            let msg = format!("Cyphertext link level packet arrived with an unrecognised NarrowWaistPacket SIZE of {}, where supported sizes are: CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}", nw_size, CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE);
            error!("{}", msg);
            return Err(anyhow!(msg));
        },
    };
    //debug!("{:?}", nw);
    if !nw.verify()? {
        let err_msg = "The manifest signature check failed when extracting the data from a NarrowWaistPacket::Response";
        error!("{}", err_msg);
        return Err(anyhow!(err_msg))
    }
    Ok((lnk_tx_pid, LinkPacket::new(reply_to, nw)))
}
pub fn deserialize_cleartext_link_packet(data: &Vec<u8>) -> Result<(PublicIdentity, LinkPacket)> {
// Link Pid
    let mut link_tx_pk = [0u8; ID_SIZE + CC_SIZE];
    link_tx_pk.clone_from_slice(&data[CLEARTEXT_LINK_TX_PK_START..CLEARTEXT_LINK_TX_PK_END]);
    //trace!("des link_tx_pk: \t\t{:?}", link_tx_pk);
    let lnk_tx_pid: PublicIdentity = PublicIdentity::from(link_tx_pk);
// Reply To Length
    let reply_to_size = &data[CLEARTEXT_LINK_REPLY_TO_SIZE_START..CLEARTEXT_LINK_REPLY_TO_SIZE_END];
    trace!("des reply_to_size: \t\t{:?}", reply_to_size);
// Narrow Waist Length
    let mut nw_size = [0u8; 2];
    nw_size.clone_from_slice(&data[CLEARTEXT_LINK_NARROW_WAIST_SIZE_START..CLEARTEXT_LINK_NARROW_WAIST_SIZE_END]);
    trace!("des nw_size: \t\t\t{:?} as_u16: {}", nw_size, u8_to_u16(nw_size));
    let nw_size: usize = u8_to_u16(nw_size) as usize;

    let reply_to: ReplyTo = deserialize_reply_to(&data[CLEARTEXT_LINK_NARROW_WAIST_SIZE_END..CLEARTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize].to_vec())?;
    trace!("des reply_to: \t\t\t{:?}", reply_to);
    let nw_start = CLEARTEXT_LINK_NARROW_WAIST_SIZE_END + reply_to_size[0] as usize;
    let nw: NarrowWaistPacket = match nw_size {
        CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
            let cleartext = &data[nw_start..nw_start + nw_size];
            trace!("des cyphertext_nw: \t\t{:?}", cleartext);
            deserialize_cyphertext_narrow_waist_packet_request(&cleartext.to_vec())?
        },
        CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
            let cleartext = &data[nw_start..nw_start + nw_size];
            trace!("des cyphertext_nw: \t\t{:?}", cleartext);
            deserialize_cyphertext_narrow_waist_packet_response(&cleartext.to_vec())?
        },
        CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
            let cleartext = &data[nw_start..nw_start + nw_size];
            trace!("des cleartext_nw: \t\t{:?}", cleartext);
            deserialize_cleartext_narrow_waist_packet_request(&cleartext.to_vec())?
        },
        CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
            let cleartext = &data[nw_start..nw_start + nw_size];
            deserialize_cleartext_narrow_waist_packet_response(&cleartext.to_vec())?
        },
        _ => {
            let msg = format!("Cleartext link level packet arrived with an unrecognised NarrowWaistPacket SIZE of {}, where supported sizes are: CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}", nw_size, CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE);
            error!("{}", msg);
            return Err(anyhow!(msg));
        },
    };
    Ok((lnk_tx_pid, LinkPacket::new(reply_to, nw)))
}
pub fn deserialize_link_packet(data: &Vec<u8>, link_id: LinkId) -> Result<(PublicIdentity, LinkPacket)> {
    match link_id.rx_pid()? {
        Some(_) => {
            deserialize_cyphertext_link_packet(data, link_id)
        },
        None => {
            deserialize_cleartext_link_packet(data)
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_u16_to_fro_u8() {
        let actual: u16 = u16::MIN;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u16 = 1;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u16 = u16::MAX;
        let expected: u16 = u8_to_u16(u16_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_bfi_to_fro_u8() {
        let actual: BFI = [0u16; BLOOM_FILTER_INDEX_ELEMENT_LENGTH];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: BFI = [0, 1, 2, 3];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: BFI = [u16::MAX, u16::MAX, u16::MAX, u16::MAX];
        let expected: BFI = u8_to_bfi(bfi_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
    #[test]
    fn test_u64_to_fro_u8() {
        let actual: u64 = 0;
        let expected: u64 = u8_to_u64(u64_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u64 = u64::MAX/2;
        let expected: u64 = u8_to_u64(u64_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);

        let actual: u64 = u64::MAX;
        let expected: u64 = u8_to_u64(u64_to_u8(actual));
        println!("expected: {:?}, actual: {:?}", expected, actual);
        assert_eq!(expected, actual);
    }
}
