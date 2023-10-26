use std::{time::Duration, fmt::Display, str::FromStr};

use crate::{serial::packet::{SerialPacket, Manufacturer}, utils::DurationUtils};

use super::{DataParseError, Data};

#[derive(Debug)]
pub struct VentilatorDataPacket {
    pub power: Option<bool>,
    pub fan_speed: Option<VentilatorFanSpeed>,
    pub mode: Option<VentilatorMode>,
    pub setting_time: Option<Duration>,
    pub remaining_time: Option<Duration>,
}

impl Data for VentilatorDataPacket {
    fn parse<T: SerialPacket>(buf: &[u8]) -> Result<Self, super::DataParseError> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => {
                if buf.len() < 3 {
                    return Err(DataParseError::LengthTooSmall);
                }
        
                let mode = Some(
                    if buf[1] == 0x02 {
                        VentilatorMode::Off
                    } else if buf[1] >= 0x01 || buf[1] <= 0x15 {
                        VentilatorMode::Normal
                    } else if buf[1] >= 0x81 || buf[1] <= 0x95 {
                        VentilatorMode::Passthrough
                    } else {
                        return Err(DataParseError::Unsupported);
                    }
                );
        
                let power = Some(*mode.as_ref().unwrap() != VentilatorMode::Off);
                let fan_speed = Some(VentilatorFanSpeed::from_pkt::<T>(buf[2].into()));
        
                let (setting_time, remaining_time) = if buf.len() == 5 {
                    (
                        Some(Duration::from_secs(buf[3] as u64 * 60)), 
                        Some(Duration::from_secs(buf[4] as u64 * 60)),
                    )
                } else if buf.len() == 6 {
                    (
                        Some(Duration::from_secs(buf[3] as u64 * 3600 + buf[4] as u64 * 60)), 
                        Some(Duration::from_secs(buf[5] as u64 * 60))
                    )
                } else {
                    (
                        None, None
                    )
                };
        
                Ok(Self {power, mode, fan_speed, setting_time, remaining_time})
            }
        }
    }

    fn create_request<T: SerialPacket>() -> Option<Vec<u8>> { 
        match T::manufacturer() {
            Manufacturer::HyundaiHT => Some(vec![0; 2])
        }
    }

    fn to_vec<T: SerialPacket>(&self) -> Option<Vec<u8>> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => {
                if self.mode.is_none() || self.fan_speed.is_none() {
                    None
                } else {
                    let mut ret = match self.mode.as_ref().unwrap() {
                        VentilatorMode::Off => vec![0, 0x02],
                        VentilatorMode::Normal => vec![0, 0x01],
                        VentilatorMode::Passthrough => vec![0, 0x81]
                    };
        
                    ret.push(self.fan_speed.as_ref().unwrap().as_u16::<T>() as u8);
        
                    if self.setting_time.is_some() && self.remaining_time.is_some() {
                        if self.setting_time.unwrap() < Duration::from_minutes(60) {
                            ret[1] += 0x4;

                            ret.push(self.setting_time.unwrap().as_section_minutes() as u8);
                            ret.push(self.remaining_time.unwrap().as_minutes() as u8);
                        } else {
                            ret[1] += 0x14;

                            ret.push(self.setting_time.unwrap().as_hours() as u8);
                            ret.push(self.setting_time.unwrap().as_section_minutes() as u8);
                            ret.push(self.remaining_time.unwrap().as_minutes() as u8);
                        }
                    }
        
                    Some(ret)
                }
            }
        }
    }
}

impl VentilatorDataPacket {
    pub fn create_mode_modify<T: SerialPacket>(mode: &VentilatorMode) -> Option<Vec<u8>> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => Some(vec![mode.as_u16::<T>() as u8, 0])
        }
        
    }

    pub fn create_timer_modify<T: SerialPacket>(time: &Duration) -> Option<Vec<u8>> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => Some(
                if time.as_secs() < 3600 {
                    vec![0x05, time.as_section_minutes() as u8]
                } else {
                    vec![0x15, time.as_hours() as u8, time.as_section_minutes() as u8]
                }
            )
        }
    }

    pub fn create_fan_modify<T: SerialPacket>(fan_speed: &VentilatorFanSpeed) -> Option<Vec<u8>> {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => Some(vec![fan_speed.as_u16::<T>() as u8, 0])
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum VentilatorMode {
    Off,
    Normal,
    Passthrough,
}

impl Display for VentilatorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for VentilatorMode {
    fn default() -> Self {
        Self::Off
    }
}

impl FromStr for VentilatorMode {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "off" => Ok(Self::Off),
            "normal" => Ok(Self::Normal),
            "passthrough" => Ok(Self::Passthrough),
            _ => Err(())
        }
    }

    type Err = ();
}

impl VentilatorMode {
    pub fn as_u16<T: SerialPacket>(&self) -> u16 {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => match self {
                VentilatorMode::Off => 0x02,
                VentilatorMode::Normal => 0x01,
                VentilatorMode::Passthrough => 0x81,
            }
        }
    }
}


#[derive(Debug)]
pub enum VentilatorFanSpeed {
    Low,
    Medium,
    High
}

impl Display for VentilatorFanSpeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for VentilatorFanSpeed {
    fn default() -> Self {
        Self::Low
    }
}

impl FromStr for VentilatorFanSpeed {
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Self::Low),
            "1" => Ok(Self::Low),
            "medium" => Ok(Self::Medium),
            "2" => Ok(Self::Medium),
            "high" => Ok(Self::High),
            "3" => Ok(Self::High),
            _ => Err(())
        }
    }

    type Err = ();
}

impl VentilatorFanSpeed {
    pub fn from_pkt<T: SerialPacket>(value: u16) -> Self {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => match value {
                0x03 => Self::Medium,
                0x07 => Self::High,
                _ => Self::Low,
            }
        }
    }

    pub fn as_u16<T: SerialPacket>(&self) -> u16 {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => match self {
                VentilatorFanSpeed::Low => 0x01,
                VentilatorFanSpeed::Medium => 0x03,
                VentilatorFanSpeed::High => 0x07,
            }
        }
    }

    pub fn to_level(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
        }
    }
}

impl From<u64> for VentilatorFanSpeed {
    fn from(value: u64) -> Self {
        match value {
            1 => Self::Low,
            2 => Self::Medium,
            3 => Self::High,
            _ => Self::Low
        }   
    }
}