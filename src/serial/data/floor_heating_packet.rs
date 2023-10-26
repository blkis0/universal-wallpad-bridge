use crate::serial::packet::{Manufacturer::HyundaiHT, SerialPacket};

use super::{Data, DataParseError};

#[derive(Debug)]
pub struct FloorHeatingDataPacket {
    pub power: Option<bool>,
    pub target_temp: Option<u32>,
    pub current_temp: Option<u32>
}

impl Data for FloorHeatingDataPacket {
    fn parse<T: SerialPacket>(buf: &[u8]) -> Result<Self, super::DataParseError> {
        match T::manufacturer() {
            HyundaiHT => {
                if buf.len() < 3 {
                    return Err(DataParseError::LengthTooSmall);
                }
        
                return Ok(
                    Self {
                        power: Some(buf[0] == 0x01),
                        target_temp: Some(buf[2] as u32),
                        current_temp: Some(buf[1] as u32)
                    }
                )
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

impl FloorHeatingDataPacket {
    pub fn create_power_modify<T: SerialPacket>(status: bool) -> Option<Vec<u8>> {
        match T::manufacturer() {
            HyundaiHT => Some(vec![if status {1} else {4}, 0x00]),
        }
    }

    pub fn create_temp_modify<T: SerialPacket>(temperature: u32) -> Option<Vec<u8>> {
        match T::manufacturer() {
            HyundaiHT => Some(vec![temperature as u8, 0x00]),
        }
    }
}