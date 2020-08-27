use {
    crate::{
    /*
        //identity::{decrypt_identity},
        narrow_waist::{NarrowWaist},
        borsh::{BorshSerialize},
    */
        node::router::{Config},
    },
    /*
    cryptoxide::{
        sha2::{
            Sha256,
        },
        digest::Digest,
    },
    */
    anyhow::{Result},
    chain_crypto::{Ed25519, PublicKey, SecretKey},
    /*
    std::{
        collections::HashSet,
    },
    */
};
/*
pub fn add_trusted_identity(password: String, identity: Packet, addresses: Vec<String>)  {
    match identity {
        Packet::Request {..} => {},
        Packet::Response {.., data} => {
            match Data::Manifest
            let data = String::from_utf8(data).unwrap();
            if let Some((sk, pk, tc)) = decrypt_identity(password, data.to_string()) {
                println!("sk {:?}, pk {:?}, hash {:?}", sk, pk, tc);
            }
        },
    };
}
*/
pub fn new_trusted_identity(_config: &Config, _sk: &SecretKey<Ed25519>, _pk: &PublicKey<Ed25519>) -> Result<String> {
    /*
    let mut hasher = Sha256::new();
    let tcs: HashSet<String> = HashSet::new();
    let tcs_ser = tcs.try_to_vec()?;
    let signature = sk.sign(&tcs_ser);
    println!("signature = {:?}", signature);
    let _res = signature.verify(&pk, &tcs_ser);
    // need to sign this with pk first, then encrypt with signature, to ensure my packet is unique otherwise everyone's inition thing will be the same
    println!("tcs_ser = {:?}", tcs_ser);
    hasher.input(&tcs_ser);
    //hasher.input_str("[]");
    let tc_hash = hasher.result_str();
    println!("tc_hash = {:?}", tc_hash.clone());

    let tc_packet = NarrowWaist::Response {tc_hash.clone(), tcs_ser.to_vec(), 0, 0)?;

    let mut tc_path = std::path::PathBuf::from(config.data_dir.clone());
    tc_path.push(".copernica");
    tc_path.push("trusted_connections");
    let tc_path = tc_path.join(tc_hash.clone());
    println!("id = {:?}", tc_path);
    let tc_ser = tc_packet.try_to_vec()?;
    std::fs::write(tc_path, tc_ser).unwrap();
    Ok(tc_hash)
    */
    Ok("blah".to_string())
}

