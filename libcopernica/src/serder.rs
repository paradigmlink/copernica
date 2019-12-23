use {
    bincode,
    base64,
    crate::{TransportPacket},
};

pub fn serialize(t: TransportPacket) -> Vec<u8> {
    let packet: Vec<u8> = bincode::serialize(&t).unwrap();
    let packet: String = base64::encode(&packet);
    let packet: Vec<u8> = bincode::serialize(&packet).unwrap();
    packet
}

pub fn deserialize(v: Vec<u8>) -> TransportPacket {
    let packet: String = bincode::deserialize(&v).unwrap();
    let packet: Vec<u8> = base64::decode(&packet).unwrap();
    let packet: TransportPacket = bincode::deserialize(&packet).unwrap();
    packet
}


#[cfg(test)]
mod serdeser {
    use {
        super::*,
        crate::{
            narrow_waist::{NarrowWaist, mk_response_packet},
            transport::{TransportPacket, InterFace},
        },
    };

    #[test]
    fn test_serialize() {
        let expected: Vec<u8> = vec![84, 0, 0, 0, 0, 0, 0, 0, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 66, 47, 65, 65, 65, 66, 110, 66, 56, 66, 65, 65, 65, 65, 66, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 69, 65, 87, 81, 68, 53, 65, 84, 69, 66, 119, 65, 65, 65, 81, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 61, 61];

        let packet: NarrowWaist = mk_response_packet("hello".to_string(), vec![0u8; 1], 0, 0);
        let interface: InterFace = InterFace::SocketAddr("127.0.0.1:8092".parse().unwrap());
        let packet: TransportPacket = TransportPacket::new(interface, packet);
        let actual = serialize(packet);
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_deserialize() {
        let expected: NarrowWaist = mk_response_packet("hello".to_string(), vec![0u8; 1], 0, 0);
        let interface: InterFace = InterFace::SocketAddr("127.0.0.1:8092".parse().unwrap());
        let expected: TransportPacket = TransportPacket::new(interface, expected);

        let packet: Vec<u8> = vec![84, 0, 0, 0, 0, 0, 0, 0, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 66, 47, 65, 65, 65, 66, 110, 66, 56, 66, 65, 65, 65, 65, 66, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 69, 65, 87, 81, 68, 53, 65, 84, 69, 66, 119, 65, 65, 65, 81, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 65, 61, 61];
        let actual: TransportPacket = deserialize(packet);
        assert_eq!(actual, expected);
    }
}

