use super::packet::SerialPacket;

pub mod realtime_energy_packet;
pub mod floor_heating_packet;
pub mod binary_switch_packet;
pub mod ventilator_packet;

pub trait Data  {
    fn parse<T: SerialPacket>(buf: &[u8]) -> Result<Self, DataParseError> where Self: Sized;
    fn create_request<T: SerialPacket>() -> Option<Vec<u8>>;

    fn to_vec<T: SerialPacket>(&self) -> Option<Vec<u8>>;
}

#[derive(Debug)]
pub enum DataParseError {
    LengthTooSmall,
    Unsupported
}