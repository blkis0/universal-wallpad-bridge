use crate::{utils::{self, xor_checksum}, things::Channels};

use super::{Command, SerialPacket, PacketHandler, PacketParseError};

#[derive(Debug, Clone)]
pub struct HyundaiPacket {
    pub length: u8,
    pub unspecified_0x02: u8,
    
    pub device_id: u8,
    pub command: Command,
    pub device_sub_id: u8,
    pub room_id: u8,

    pub header_checksum: u8,

    pub data: Vec<u8>,
    pub data_checksum: u8,
}

impl HyundaiPacket {
    pub const PREFIX: &'static [u8] = &[0xF7];
    pub const SUFFIX: &'static [u8] = &[0xEE];

    pub fn new(device_id: u8, command: Command, device_sub_id: u8, room_id: u8, data: Vec<u8>) -> Self {
        Self { 
            length: 9 + data.len() as u8, 
            unspecified_0x02: 0x01, 

            device_id: device_id, 
            command: command, 
            device_sub_id: device_sub_id, 
            room_id: room_id, 
            
            header_checksum: 0x00, 
            
            data: data, 
            data_checksum: 0x00
        }
    }
}

impl SerialPacket for HyundaiPacket {
    fn to_vec(&self) -> Vec<u8> {
        let mut v: Vec<u8> = vec![];

        v.extend(HyundaiPacket::PREFIX);

        v.push(9 + self.data.len() as u8);
        v.push(self.unspecified_0x02);
        v.push(self.device_id);

        v.push(match self.command {
            Command::Request => 0x01,
            Command::Modify => 0x02,
            Command::Response => 0x04
        });

        v.push(self.device_sub_id);
        v.push(self.room_id);
        
        v.extend(&self.data);

        v.push(xor_checksum(&v, v.len()));

        v.extend(HyundaiPacket::SUFFIX);
    
        return v;
    }

    fn parse(buf: &[u8]) -> Result<Self, PacketParseError> {
        if buf.len() < 9 {
            return Err(PacketParseError::BufferLengthTooSmall)                                                 
        }

        if buf[0x01] as usize != buf.len() {                                                                         
            return Err(PacketParseError::SizeMismatch)
        }

        if buf[buf.len() - 2] != utils::xor_checksum(&buf, buf.len() - 2) {                              
            return Err(PacketParseError::ChecksumMismatch)
        }

        let data_buf = buf[0x07..buf.len() - 2].to_vec();
        let data_checksum = utils::xor_checksum(data_buf.as_slice(), buf.len() - 9);

        let command = match buf[0x04] {
            0x01 => Command::Request,
            0x02 => Command::Modify,
            0x04 => Command::Response,
            _ => return Err(PacketParseError::UnsupportedCommand)
        };

        Ok(Self{
            length: buf[0x01], 
            unspecified_0x02: buf[0x02], 
            device_id: buf[0x03], 
            command: command, 
            device_sub_id: buf[0x05], 
            room_id: buf[0x06], 
            header_checksum: buf[buf.len() - 2] ^ data_checksum, 
            data: data_buf, 
            data_checksum: data_checksum
        })
    }

    fn data(&self) -> &Vec<u8> {
        &self.data
    }

    fn mut_data(&mut self) -> &mut Vec<u8> {
        self.data.as_mut()
    }

    fn manufacturer() -> super::Manufacturer { super::Manufacturer::HyundaiHT }

    fn command(&self) -> Command {
        self.command
    }

    fn is_correct_response(&self, response: &Self) -> bool {
        self.command == Command::Response || (response.command == Command::Response && self.device_id == response.device_id && self.device_sub_id == response.device_sub_id && self.room_id == response.room_id)
    }

    fn length_from_buffer(buf: &[u8]) -> Option<usize> {
        if buf.len() < 2 { None }
        else { Some(buf[1] as usize) }
    }

    fn baud_rate() -> u32 where Self: Sized {
        9600
    }
}


pub struct HyundaiPacketHandler {
    pub device_id: Option<u8>,
    pub device_sub_id: Option<u8>,
    pub room_id: Option<u8>,

    pub callback: Box<fn(&HyundaiPacket, &Channels<HyundaiPacket>)>,
    pub is_primary: bool,
    pub chaining: bool
}

impl PacketHandler<HyundaiPacket> for HyundaiPacketHandler {
    fn handle(&self, packet: &HyundaiPacket, channels: &Channels<HyundaiPacket>) -> bool {
        if 
            (self.device_id.is_none() || self.device_id.unwrap() == packet.device_id) &&
            (self.device_sub_id.is_none() || self.device_sub_id.unwrap() == packet.device_sub_id) &&
            (self.room_id.is_none() || self.room_id.unwrap() == packet.room_id)
        {
            (self.callback)(packet, channels);
            return true;
        }

        return false;
    }

    fn chaining(&self) -> bool { self.chaining }

    fn is_primary(&self) -> bool {
        self.is_primary
    }
}
