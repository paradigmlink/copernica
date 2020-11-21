use {
    crate::{
        constants,
        hbfi::HBFI,
        link::{LinkId, ReplyTo},
    },
    serde::{Deserialize, Serialize},
    std::fmt,
    serde_big_array::{big_array},
    keynesis::{PublicIdentity, PrivateIdentity, Signature},
    anyhow::{anyhow, Result},
    rand_core::{CryptoRng, RngCore},
    rand::Rng,
    log::{debug},
    cryptoxide::{chacha20poly1305::{ChaCha20Poly1305}},
    nom::{
        IResult,
        bytes::complete::{tag, take_while_m_n},
        combinator::map_res,
        sequence::tuple,
    },
};


#[derive(Debug,PartialEq)]
pub struct Color {
    pub red:   u8,
    pub green: u8,
    pub blue:  u8,
}

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_digit(16)
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(
        take_while_m_n(2, 2, is_hex_digit),
        from_hex
    )(input)
}

fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

    Ok((input, Color { red, green, blue }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packets::{NarrowWaistPacket, LinkPacket},
    };
    use keynesis::{PrivateIdentity, Seed};
    use nom::{
        IResult,
        bytes::complete::{tag, take_while_m_n},
        combinator::map_res,
        sequence::tuple
    };

    #[test]
    fn parse_color() {
        println!("{:?}", hex_color("#2F14DF").unwrap());
        assert_eq!(hex_color("#2F14DF"), Ok(("", Color {
            red: 47,
            green: 20,
            blue: 223,
        })));
    }

    #[test]
    fn parse_packet() {
        // https://gafferongames.com/post/packet_fragmentation_and_reassembly
        let mut rng = rand::thread_rng();
        let response_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let response_pid = response_sid.public_id();

        let request_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let request_pid = response_sid.public_id();


        let hbfi = HBFI::new(response_pid.clone(), Some(request_pid), "app", "m0d", "fun", "arg").unwrap();
        let nw: NarrowWaistPacket = NarrowWaistPacket::request(hbfi.clone()).unwrap();
        let expected_data = vec![0; 600];
        let offset = 0;
        let total = 1;
        let nw: NarrowWaistPacket = nw.transmute(response_sid.clone(), expected_data.clone(), offset, total).unwrap();

        let link_sid = PrivateIdentity::from_seed(Seed::generate(&mut rng));
        let link_pid = link_sid.public_id();

        let reply_to: ReplyTo = ReplyTo::UdpIp("127.0.0.1:50000".parse().unwrap());
        let lp: LinkPacket = LinkPacket::new(link_pid, reply_to, nw);
        println!("{:?}", lp);
        let lp_ser = bincode::serialize(&lp).unwrap();
        let lp_ser_len = lp_ser.len();
        println!("must be less than 1472, current length: {}", lp_ser_len);
        let lt1472 = if lp_ser_len <= 1472 { true } else { false };
        assert_eq!(true, lt1472);
    }
}
