use crate::serial::packet::{Manufacturer, SerialPacket};

use super::{Data, DataParseError};

#[derive(Debug)]
pub struct BinarySwitchDataPacket {
    pub status: Option<bool>
}

impl Data for BinarySwitchDataPacket {
    fn parse<T: SerialPacket>(buf: &[u8]) -> Result<Self, super::DataParseError> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => {
                if buf.len() < 1 {
                    return Err(DataParseError::LengthTooSmall);
                }
                
                Ok( Self {
                    status: Some(buf[0] == 0x01 || buf[0] == 0x04) // 0x01: Light, 0x04: Gas
                })
            }
        }
    }

    fn create_request<T: SerialPacket>() -> Option<Vec<u8>> { 
        match T::manufacturer() {
            Manufacturer::HyundaiHT => Some(vec![0; 2]),
        }
    }

    fn to_vec<T: SerialPacket>(&self) -> Option<Vec<u8>> {
        todo!()
    }
}


impl BinarySwitchDataPacket {
    pub fn create_modify<T: SerialPacket>(value: bool) -> Option<Vec<u8>> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => Some(vec![if value {0x01} else {0x02}, 0x00]),
        }
    }

    pub fn create_gas_valve_modify<T: SerialPacket>(value: bool) -> Option<Vec<u8>> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => Some(vec![if value {0x04} else {0x03}, 0x00]),
        }
    }
}