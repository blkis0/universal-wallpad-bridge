use crate::serial::packet::{Manufacturer::HyundaiHT, SerialPacket};

use super::{Data, DataParseError};

fn hex_usage_to_dec(buf: &[u8], offset: usize) -> String {
    return format!("{:02x}{:02x}", buf[offset], buf[offset + 1])
}

#[derive(Debug)]
pub struct RealtimeEnergyDataPacket {
    pub electric: Option<u32>,
    pub water: Option<u32>,
    pub gas: Option<u32>
}

impl Data for RealtimeEnergyDataPacket {
    fn parse<T: SerialPacket>(buf: &[u8]) -> Result<Self, super::DataParseError> {
        match T::manufacturer() {
            HyundaiHT => {
                if buf.len() < 17 {
                    return Err(DataParseError::LengthTooSmall);
                }

                Ok( Self {
                    electric: Some(hex_usage_to_dec(&buf, 3).parse::<u32>().unwrap_or_default()),
                    water: Some(hex_usage_to_dec(&buf, 15).parse::<u32>().unwrap_or_default()),
                    gas: Some(hex_usage_to_dec(&buf, 11).parse::<u32>().unwrap_or_default())
                })
            }
        }
    }

    fn create_request<T: SerialPacket>() -> Option<Vec<u8>> { 
        match T::manufacturer() {
            HyundaiHT => Some(vec![0; 2]),
        }
    }

    fn to_vec<T: SerialPacket>(&self) -> Option<Vec<u8>> {
        todo!()
    }
}