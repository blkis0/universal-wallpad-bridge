use std::{marker::PhantomData, time::Duration};


use rumqttd::{local::LinkTx, protocol::Publish};


use crate::{serial::{packet::{hyundai::{HyundaiPacket, HyundaiPacketHandler}, SerialPacket, Command, PacketHandler}, data::{Data, ventilator_packet::{VentilatorDataPacket, VentilatorMode, VentilatorFanSpeed}}}, utils::{self, DurationUtils}};

use super::{Thing, Channels};

#[derive(Clone)]
pub struct Ventilator<T: SerialPacket> {
    _marker: PhantomData<T>,
}

static mut LATEST_MODE: VentilatorMode = VentilatorMode::Normal;
static mut LATEST_STATE: bool = false;

impl Ventilator<HyundaiPacket> {
    fn on_response(packet: &HyundaiPacket, channels: &Channels<HyundaiPacket>) {
        match VentilatorDataPacket::parse::<HyundaiPacket>(&packet.data) {
            Ok(data) => {
                println!("{:?}", data);

                match utils::link_tx_lock(&channels.link_tx.clone()) {
                    Some(mut link_tx) => {
                        
                            let mut result = unsafe {
                                LATEST_STATE = data.power.unwrap_or_default();
                            
                                link_tx.publish(format!("ventilator/power"), LATEST_STATE.to_string())
                            }
                            .and(link_tx.publish(format!("ventilator/mode"), data.mode.as_ref().unwrap_or(&VentilatorMode::Off).to_string()))
                            .and(link_tx.publish(format!("ventilator/fan_speed"), data.fan_speed.as_ref().unwrap_or(&VentilatorFanSpeed::Low).to_level().to_string()));
                            
                            unsafe {
                                if LATEST_STATE {
                                    LATEST_MODE = data.mode.unwrap();
                                }
                            }
                            
                            /*
                            let mode = data.mode.unwrap();

                            if mode != VentilatorMode::Normal {
                                match utils::serial_tx_lock(&channels.serial_tx.clone()) {
                                    Some(link_tx) => {
                                        let mut pkt = packet.clone();

                                        pkt.data[1] = VentilatorMode::Normal.as_u16::<HyundaiPacket>() as u8 + (pkt.data[1] - mode.as_u16::<HyundaiPacket>() as u8);

                                        match link_tx.send((pkt, 2)) {
                                            Ok(_) => (),
                                            Err(e) => eprintln!("{:?}", e)
                                        }
                                    }
                                    _ => ()
                                }
                            }
                            */

                            
                            result = result.and(link_tx.publish(format!("ventilator/timer/status"), data.setting_time.is_some().to_string()));

                            if data.setting_time.is_some() {
                                result = result.and(link_tx.publish(format!("ventilator/timer"), data.setting_time.as_ref().unwrap_or(&Duration::ZERO).as_minutes().to_string()))    
                            }

                            result = result.and(link_tx.publish(format!("ventilator/timer/remaining"), if data.remaining_time.is_some() {
                                data.remaining_time.as_ref().unwrap_or(&Duration::ZERO).as_minutes().to_string()
                            } else {
                                "0".into()
                            }));
                            
                                
                            if result.is_err() {
                                eprintln!("{:?}", result.unwrap_err());
                            }
                    },
                    _ => ()
                }
            },
            Err(e) => eprintln!("{:?}", e)
        }
    }

    fn set_power(value: bool, channels: &Channels<HyundaiPacket>) {
        Self::set_mode(if value { unsafe { match &LATEST_MODE {
            VentilatorMode::Off => &VentilatorMode::Normal,
            VentilatorMode::Normal => &VentilatorMode::Passthrough,
            VentilatorMode::Passthrough => &VentilatorMode::Normal,
        } } } else {
            &VentilatorMode::Off
        }, channels)
    }

    fn set_mode(value: &VentilatorMode, channels: &Channels<HyundaiPacket>) {
                let mut p = HyundaiPacket::new(
                    0x2B, Command::Modify, 0x40, 0x11,
                    VentilatorDataPacket::create_mode_modify::<HyundaiPacket>(&VentilatorMode::Normal).unwrap()
                );
                
                unsafe {
                    if !LATEST_STATE && (*value != VentilatorMode::Normal || *value != VentilatorMode::Off) {
                        match channels.serial_tx.send(p.clone()) {
                            Ok(_) => (),
                            Err(e) => eprintln!("{:?}", e)
                        }
                    }
                }

                p.data = VentilatorDataPacket::create_mode_modify::<HyundaiPacket>(&value).unwrap();

                match channels.serial_tx.send(p) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e)
                }
    }

    fn set_timer(value: &Duration, channels: &Channels<HyundaiPacket>) {
        unsafe {
            if !LATEST_STATE {
                Self::set_power(true, channels);
            } 
        }
                let p = HyundaiPacket::new(
                    0x2B, Command::Modify, 0x40, 0x11,
                    VentilatorDataPacket::create_timer_modify::<HyundaiPacket>(value).unwrap()
                );

                match channels.serial_tx.send(p) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e)
                }
    }

    fn set_fan_speed(value: &VentilatorFanSpeed, channels: &Channels<HyundaiPacket>) {
        
                let p = HyundaiPacket::new(
                    0x2B, Command::Modify, 0x42, 0x11,
                    VentilatorDataPacket::create_fan_modify::<HyundaiPacket>(value).unwrap()
                );

                match channels.serial_tx.send(p) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e)
                }
    }

    pub fn new() -> Box<dyn Thing<HyundaiPacket> + Send> {
        Box::new(Self{ _marker: PhantomData })
    }
}


impl Thing<HyundaiPacket> for Ventilator<HyundaiPacket> {
    fn handler(&self) -> Box<dyn PacketHandler<HyundaiPacket> + Send> {
        Box::new(HyundaiPacketHandler {
            device_id: Some(0x2B),
            device_sub_id: None,
            room_id: Some(0x11),
            callback: Box::new(|pk, ch| { 
                match pk.command {
                    Command::Response => Self::on_response(pk, ch),
                    _ => ()
                }
            }),
            chaining: true,
            is_primary: true
        })
    }

    fn topic_handler(&self) -> fn(&Publish, &Channels<HyundaiPacket>, &Option<&Channels<HyundaiPacket>>) {
        |pk, ch, _ch2| {
            let topic = String::from_utf8_lossy(&pk.topic);

            if topic.len() >= 12 {
                if topic.starts_with("ventilator/") {
                    if !topic.ends_with("/set") {
                        return;
                    }

                    let method = &topic[11..(topic.len() - 4)];
                    let payload = String::from_utf8_lossy(&pk.payload);
                    println!("{}",payload);
                    match method {
                        "power" => {
                            Self::set_power(payload.parse::<bool>().unwrap_or_default(), ch);
                        },
                        "mode" => {
                            Self::set_mode(&payload.parse::<VentilatorMode>().unwrap_or_default(), ch);
                        },
                        "fan_speed" => {
                            if payload == "0" {
                                Self::set_power(false, ch);
                            } else {
                                Self::set_fan_speed(&payload.parse::<VentilatorFanSpeed>().unwrap_or_default(), ch);
                            }
                        },
                        "timer" => {
                            Self::set_timer(&Duration::from_minutes(payload.parse::<u64>().unwrap_or_default()), ch);
                        },
                        _ => {
                           /*match utils::serial_tx_lock(&ch.serial_tx.clone()) {
                                Some(link_tx) => {
                                    let mut p = HyundaiPacket::new(
                                        0x2B, Command::Modify, 0x40, 0x11,
                                        vec![1, payload.parse::<u8>().unwrap()]
                                    );
                    
                                    match link_tx.send(p) {
                                        Ok(_) => (),
                                        Err(e) => eprintln!("{:?}", e)
                                    }
                                }
                                _ => ()
                            }
                            */
                        }
                    }
                }
            }
        }
    }

    fn subscribe(&self, link_tx: &mut LinkTx) {
        link_tx.subscribe("ventilator/+/set").unwrap();
    }

    fn new() -> Box<dyn Thing<HyundaiPacket> + Send> where Self: Sized {
        Box::new(Ventilator::<HyundaiPacket> {
            _marker: PhantomData,
        })
    }

    fn task(&self) -> Option<fn(&Channels<HyundaiPacket>, &Option<&Channels<HyundaiPacket>>)> {
        None
    }
}
