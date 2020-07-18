use {
    crate::{
        TransportPacket,
        borsh::{BorshDeserialize, BorshSerialize},
    },
    anyhow::{Result},
};

pub fn serialize(packet: &TransportPacket) -> Result<Vec<u8>> {
    let packet: Vec<u8> = packet.try_to_vec()?;
    Ok(packet)
}

pub fn deserialize(packet: &[u8]) -> Result<TransportPacket> {
    let packet: TransportPacket = TransportPacket::try_from_slice(&packet)?;
    Ok(packet)
}
