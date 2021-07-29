use {
    crate::{
        hbfi::HBFI,
        manifest, generate_nonce,
        ResponseData, Nonce,
        PrivateIdentityInterface,
        Signature,
        constants::*, Tag,
        PublicIdentity, LinkId,
    },
    core::hash::{Hash},
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    std::{
        fmt,
    },
    anyhow::{anyhow, Result},
    log::{error},
};
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NarrowWaistPacket {
    Request {
        hbfi: HBFI,
        nonce: Nonce,
    },
    Response {
        hbfi: HBFI,
        nonce: Nonce,
        signature: Signature,
        data: ResponseData,
    },
}
impl NarrowWaistPacket {
    pub fn request(hbfi: HBFI) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let nonce: Nonce = generate_nonce(&mut rng);
        Ok(NarrowWaistPacket::Request { hbfi, nonce })
    }
    pub fn response(response_sid: PrivateIdentityInterface, hbfi: HBFI, data: Vec<u8>) -> Result<Self> {
        if hbfi.response_pid != response_sid.public_id() {
            let msg = "The Request's Response Public Identity doesn't match the Public Identity used to sign or encypt the Response";
            error!("{}", msg);
            return Err(anyhow!(msg));
        }
        let mut rng = rand::thread_rng();
        let hbfi = hbfi.clone();
        let nonce: Nonce = generate_nonce(&mut rng);
        let data = ResponseData::insert(response_sid.clone(), hbfi.request_pid.clone(), data, nonce.clone())?;
        let manifest = manifest(&data, &hbfi, &nonce);
        let response_signkey = response_sid.signing_key();
        let signature = response_signkey.sign(manifest);
        Ok(NarrowWaistPacket::Response { hbfi, nonce, data, signature })
    }
    pub fn verify(&self) -> Result<bool> {
        match self {
            NarrowWaistPacket::Request {..} => {
                return Ok(true)
            },
            NarrowWaistPacket::Response { data, hbfi, signature, nonce} => {
                let manifest = manifest(&data, &hbfi, &nonce);
                let verify_key = hbfi.response_pid.verify_key()?;
                let verified = verify_key.verify(&signature, manifest);
                return Ok(verified);
            },
        }
    }
    pub fn data(&self, request_sid: PrivateIdentityInterface) -> Result<Vec<u8>> {
        match self {
            NarrowWaistPacket::Request {..} => {
                let err_msg = "No data in a NarrowWaistPacket::Request";
                error!("{}", err_msg);
                return Err(anyhow!(err_msg))
            },
            NarrowWaistPacket::Response { data, hbfi, nonce, ..}=> {
                return Ok(data.extract(request_sid, hbfi.request_pid.clone(), hbfi.response_pid.clone(), nonce.clone())?)

            },
        }
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![];
        match self {
            NarrowWaistPacket::Request { hbfi, nonce } => {
                buf.extend_from_slice(&nonce.0);
                buf.extend_from_slice(&hbfi.as_bytes());
            },
            NarrowWaistPacket::Response { hbfi, signature, nonce, data } => {
                buf.extend_from_slice(signature.as_ref());
                buf.extend_from_slice(&nonce.0);
                buf.extend_from_slice(&hbfi.as_bytes());
                buf.extend_from_slice(&data.as_bytes());
            },
        }
        buf
    }
    pub fn from_cleartext_bytes(data: &[u8]) -> Result<Self> {
        let nw_size = data.len();
        let nw: NarrowWaistPacket = match nw_size {
            CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&data[0..NONCE_SIZE]);
                let hbfi: HBFI = HBFI::from_cyphertext_bytes(&data[NONCE_SIZE..NONCE_SIZE+CYPHERTEXT_HBFI_SIZE])?;
                NarrowWaistPacket::Request { hbfi, nonce }
            },
            CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
                let mut signature = [0u8; Signature::SIZE];
                signature.clone_from_slice(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
                let signature: Signature = Signature::from(signature);
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
                let hbfi_end = CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END + CYPHERTEXT_HBFI_SIZE;
                let response_data_end = hbfi_end + CYPHERTEXT_RESPONSE_DATA_SIZE;
                let hbfi: HBFI = HBFI::from_cyphertext_bytes(&data[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END..hbfi_end])?;
                let data: ResponseData = ResponseData::from_cyphertext_bytes(&data[hbfi_end..response_data_end].to_vec())?;
                NarrowWaistPacket::Response { hbfi, signature, nonce, data }
            },
            CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&data[0..NONCE_SIZE]);
                let hbfi: HBFI = HBFI::from_cleartext_bytes(&data[NONCE_SIZE..NONCE_SIZE+CLEARTEXT_HBFI_SIZE])?;
                NarrowWaistPacket::Request { hbfi, nonce }
            },
            CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
                let mut signature = [0u8; Signature::SIZE];
                signature.clone_from_slice(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
                let signature: Signature = Signature::from(signature);
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
                let hbfi_end = CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END + CLEARTEXT_HBFI_SIZE;
                let response_data_end = hbfi_end + CLEARTEXT_RESPONSE_DATA_SIZE;
                let hbfi: HBFI = HBFI::from_cleartext_bytes(&data[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END..hbfi_end])?;
                let data: ResponseData = ResponseData::from_cleartext_bytes(&data[hbfi_end..response_data_end].to_vec())?;
                NarrowWaistPacket::Response { hbfi, signature, nonce, data }
            },
            _ => {
                let msg = format!("Cleartext link level packet arrived with an unrecognised NarrowWaistPacket SIZE of {}, where supported sizes are: CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}", nw_size, CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE);
                error!("{}", msg);
                return Err(anyhow!(msg));
            },
        };
        if !nw.verify()? {
            let err_msg = "NarrowWaistPacket::from_cleartext_bytes signature check failed";
            error!("{}", err_msg);
            return Err(anyhow!(err_msg))
        }
        Ok(nw)
    }
    pub fn from_cyphertext_bytes(data: &[u8], link_id: LinkId, link_nonce: Nonce, lnk_tx_pid: PublicIdentity, link_tag: Tag) -> Result<Self> {
        let nw_size = data.len();
        let shared_secret = link_id.shared_secret(link_nonce.clone(), lnk_tx_pid.clone())?;
        let mut ctx = ChaCha20Poly1305::new(&shared_secret.as_ref(), &link_nonce.0, &[]);
        drop(shared_secret);
        let mut decrypted = vec![0u8; nw_size];
        let encrypted = &data[..nw_size];
        if !ctx.decrypt(encrypted, &mut decrypted, &link_tag.0) {
            let err_msg = "Failed to decrypt NarrowWaistPacket";
            error!("{}", err_msg);
            return Err(anyhow!(err_msg))
        };
        let nw: NarrowWaistPacket = match nw_size {
            CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&decrypted[0..NONCE_SIZE]);
                let hbfi: HBFI = HBFI::from_cyphertext_bytes(&decrypted[NONCE_SIZE..NONCE_SIZE+CYPHERTEXT_HBFI_SIZE])?;
                NarrowWaistPacket::Request { hbfi, nonce }
            },
            CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
                let mut signature = [0u8; Signature::SIZE];
                signature.clone_from_slice(&decrypted[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
                let signature: Signature = Signature::from(signature);
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&decrypted[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
                let hbfi_end = CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END + CYPHERTEXT_HBFI_SIZE;
                let response_data_end = hbfi_end + CYPHERTEXT_RESPONSE_DATA_SIZE;
                let hbfi: HBFI = HBFI::from_cyphertext_bytes(&decrypted[CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END..hbfi_end])?;
                let data: ResponseData = ResponseData::from_cyphertext_bytes(&decrypted[hbfi_end..response_data_end].to_vec())?;
                NarrowWaistPacket::Response { hbfi, signature, nonce, data }
            },
            CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE => {
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&decrypted[0..NONCE_SIZE]);
                let hbfi: HBFI = HBFI::from_cleartext_bytes(&decrypted[NONCE_SIZE..NONCE_SIZE+CLEARTEXT_HBFI_SIZE])?;
                NarrowWaistPacket::Request { hbfi, nonce }
            },
            CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE => {
                let mut signature = [0u8; Signature::SIZE];
                signature.clone_from_slice(&decrypted[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIG_END]);
                let signature: Signature = Signature::from(signature);
                let mut nonce = Nonce([0u8; NONCE_SIZE]);
                nonce.0.clone_from_slice(&decrypted[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_START..CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END]);
                let hbfi_end = CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END + CLEARTEXT_HBFI_SIZE;
                let response_data_end = hbfi_end + CLEARTEXT_RESPONSE_DATA_SIZE;
                let hbfi: HBFI = HBFI::from_cleartext_bytes(&decrypted[CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_NONCE_END..hbfi_end])?;
                let data: ResponseData = ResponseData::from_cleartext_bytes(&decrypted[hbfi_end..response_data_end].to_vec())?;
                NarrowWaistPacket::Response { hbfi, signature, nonce, data }
            },
            _ => {
                let msg = format!("Cyphertext link level packet arrived with an unrecognised NarrowWaistPacket SIZE of {}, where supported sizes are: CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE {}, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE {}", nw_size, CYPHERTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CYPHERTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_REQUEST_SIZE, CLEARTEXT_NARROW_WAIST_PACKET_RESPONSE_SIZE);
                error!("{}", msg);
                return Err(anyhow!(msg));
            },
        };
        if !nw.verify()? {
            let err_msg = "NarrowWaistPacket::from_cyphertext_bytes signature check failed";
            error!("{}", err_msg);
            return Err(anyhow!(err_msg))
        }
        Ok(nw)
    }
}
impl fmt::Debug for NarrowWaistPacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            NarrowWaistPacket::Request { hbfi, .. } => write!(f, "NW REQ {:?}", hbfi),
            NarrowWaistPacket::Response {
                hbfi,
                signature,
                nonce,
                ..
            } => write!(f, "NW RES {:?} {} {:?}", hbfi, signature, nonce),
        }
    }
}
