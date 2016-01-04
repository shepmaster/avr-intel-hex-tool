use std::fs::File;
use std::str::{self, FromStr};
use std::io::prelude::*;

#[derive(Debug, Clone)]
enum Record {
    Data { address: u16, data: Vec<u8> },
    EndOfFile,
    // ExtendedSegmentAddress { base_address: u16 },
    // StartSegmentAddress { cs: u16, ip: u16 },
    // ExtendedLinearAddress { base_address: u16 },
    // StartLinearAddress { eip: u32 },
}

#[derive(Debug, Copy, Clone)]
enum Error {
    IncorrectInitialCharacter,
    OddNumberOfBytes,
    NonHexData,
    InvalidChecksum,
    IncompleteLine,
    MismatchedDataLength,
}

impl FromStr for Record {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Error::*;
        use Record::*;

        if ! s.starts_with(":") {
            return Err(IncorrectInitialCharacter);
        }

        let bytes = s[1..].as_bytes();
        if bytes.len() % 2 != 0 {
            return Err(OddNumberOfBytes);
        }

        let raw_data: Result<Vec<_>, _> = bytes.chunks(2).map(|c| {
            let s = try!(str::from_utf8(c).map_err(|_| NonHexData));
            u8::from_str_radix(s, 16).map_err(|_| NonHexData)
        }).collect();
        let raw_data = try!(raw_data);

        let sum = raw_data.iter().fold(0u8, |a, &v| a.wrapping_add(v));
        if sum != 0 {
            return Err(InvalidChecksum);
        }

        if raw_data.len() < 5 {
            return Err(IncompleteLine);
        }

        let len = raw_data[0];
        let address = (raw_data[1] as u16) << 8 | raw_data[2] as u16;
        let kind = raw_data[3];
        let data = &raw_data[4..raw_data.len()-1];

        if len as usize != data.len() {
            return Err(MismatchedDataLength);
        }

        let r = match kind {
            0x00 => Data { address: address, data: data.into() },
            0x01 => EndOfFile,
            _ => unimplemented!(),
            // 0x02 => ExtendedSegmentAddress,
            // 0x03 => StartSegmentAddress,
            // 0x04 => ExtendedLinearAddress,
            // 0x05 => StartLinearAddress,
        };

        Ok(r)
    }
}

fn main() {
    let mut f = File::open("/tmp/hex").expect("Can't open");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("Could not read");

    let data: Result<Vec<Record>, _> = s.lines().map(|s| s.parse()).collect();

    println!("{:?}", data);
}
