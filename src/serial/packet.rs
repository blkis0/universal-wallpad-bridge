use std::{fmt::Debug, str::FromStr};



use crate::things::Channels;

pub mod hyundai;

pub trait SerialPacket: Clone + Debug + Send {
    fn to_vec(&self) -> Vec<u8>;
    fn parse(buf: &[u8]) -> Result<Self, PacketParseError> where Self: Sized;

    fn data(&self) -> &Vec<u8>;
    fn mut_data(&mut self) -> &mut Vec<u8>;

    fn manufacturer() -> Manufacturer where Self: Sized;

    fn command(&self) -> Command;

    fn is_correct_response(&self, response: &Self) -> bool;

    fn length_from_buffer(buf: &[u8]) -> Option<usize>;
    fn baud_rate() -> u32 where Self: Sized;
}

pub trait PacketHandler<T: SerialPacket + ?Sized> {
    fn handle(&self, packet: &T, channels: &Channels<T>) -> bool;
    fn chaining(&self) -> bool;
    fn is_primary(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Command {
    Request = 0x01,
    Modify = 0x02,
    Response = 0x03
}

#[derive(Debug)]
pub enum PacketParseError {
    BufferLengthTooSmall,
    SizeMismatch,
    ChecksumMismatch,
    UnsupportedCommand
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Manufacturer {
    #[clap(name = "hyundai_ht")]
    HyundaiHT
}

impl FromStr for Manufacturer {
    fn from_str(s: &str) -> Result<Self, ()> {
        match s.to_lowercase().as_str() {
            "hyundaiht" => Ok(Self::HyundaiHT),
            _ => Err(())
        }
    }

    type Err = ();
}