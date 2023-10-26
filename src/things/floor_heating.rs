use std::marker::PhantomData;

use rumqttd::{local::LinkTx, protocol::Publish};

use crate::{serial::{packet::{hyundai::{HyundaiPacket, HyundaiPacketHandler}, SerialPacket, Command, PacketHandler}, data::{floor_heating_packet::FloorHeatingDataPacket, Data, DataParseError}}, utils};

use super::{Thing, Channels};

#[derive(Clone)]
pub struct FloorHeating<T: SerialPacket> where FloorHeating<T>: Thing<T> {
    pub _marker: PhantomData<T>,
}

//const FULL_REQUEST_PACKET: &[u8] = &[0xF7, 0x0B, 0x01, 0x18, 0x01, 0x45, 0x10, 0x00, 0x00, 0xB1, 0xEE];

impl FloorHeating<HyundaiPacket> {
    fn send_data(room_id: u8, data: &[u8], link_tx: &mut LinkTx, output: bool) {
        match FloorHeatingDataPacket::parse::<HyundaiPacket>(data) {
            Ok(data) => {
                if output {
                    println!("{:?}", data);
                }
                
                let result = link_tx
                    .publish(format!("heating/{}/power", room_id), data.power.unwrap().to_string())
                    .and(link_tx.publish(format!("heating/{}/temp/current", room_id), data.current_temp.unwrap().to_string()))
                    .and(link_tx.publish(format!("heating/{}/temp/target", room_id), data.target_temp.unwrap().to_string()));
                
        
                if result.is_err() {
                    eprintln!("{:?}", result.unwrap_err());
                }
            },
            Err(e) => eprintln!("{:?}", e)
        }
    }


    fn on_response(packet: &HyundaiPacket, channels: &Channels<HyundaiPacket> ) {
        match utils::link_tx_lock(&channels.link_tx.clone()) {
            Some(mut link_tx) => {
                Self::send_data(packet.room_id - 16, &packet.data[1..4], &mut link_tx, true);
            },
            _ => ()
        }
    }

    fn on_full_response(packet: &HyundaiPacket, channels: &Channels<HyundaiPacket>) {
        if packet.data.len() < 13 {
            eprintln!("{:?}", DataParseError::LengthTooSmall);
            return;
        }

        match utils::link_tx_lock(&channels.link_tx.clone()) {
            Some(mut link_tx) => {
                Self::send_data(0, &packet.data[1..4], &mut link_tx, false);
                Self::send_data(1, &packet.data[4..7], &mut link_tx, false);
                Self::send_data(2, &packet.data[7..10], &mut link_tx, false);
                Self::send_data(3, &packet.data[10..13], &mut link_tx, false);
            },
            _ => ()
        }
    }

    
    fn set_temp(room_id: u8, temp: u8, channels: &Channels<HyundaiPacket>) {
                let p = HyundaiPacket::new(
                    0x18, Command::Modify, 0x45, 0x11 + room_id,
                    FloorHeatingDataPacket::create_temp_modify::<HyundaiPacket>(temp.into()).unwrap()
                );

                match channels.serial_tx.send(p) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e)
                }

    }

    fn set_power(room_id: u8, power: bool, channels: &Channels<HyundaiPacket>) {
                let p = HyundaiPacket::new(
                    0x18, Command::Modify, 0x46, 0x11 + room_id,
                    FloorHeatingDataPacket::create_power_modify::<HyundaiPacket>(power).unwrap()
                );

                match channels.serial_tx.send(p) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e)
                }

    }

    fn set_mode(room_id: u8, mode: &str, channels: &Channels<HyundaiPacket>) {

                let p = HyundaiPacket::new(
                    0x18, Command::Modify, 0x46, 0x11 + room_id,
                    FloorHeatingDataPacket::create_power_modify::<HyundaiPacket>(match mode {
                        "heat" => true,
                        _ => false
                    }).unwrap()
                );

                match channels.serial_tx.send(p) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e)
                }

    }
}


impl Thing<HyundaiPacket> for FloorHeating<HyundaiPacket> {
    fn handler(&self) -> Box<dyn PacketHandler<HyundaiPacket> + Send> {
        Box::new(HyundaiPacketHandler {
            device_id: Some(0x18),
            device_sub_id: None,
            room_id: None,
            callback: Box::new(|pk, ch| { 
                match pk.command {
                    Command::Response => {
                        match pk.device_sub_id {
                            0x45 => {
                                if pk.room_id == 0x10 {
                                    Self::on_full_response(pk, ch);
                                } else {
                                    Self::on_response(pk, ch);
                                }
                            },
                            0x46 => {
                                Self::on_response(pk, ch);
                            },
                            _ => ()
                        }
                    },
                    _ => ()
                }
            }),
            chaining: false,
            is_primary: true
        })
    }

    fn task(&self) -> Option<fn(&Channels<HyundaiPacket>, &Option<&Channels<HyundaiPacket>>)> {
        None
    }

    fn topic_handler(&self) -> fn(&Publish, &Channels<HyundaiPacket>, &Option<&Channels<HyundaiPacket>>) {
        |pk, ch, _ch2| {
                    let topic = String::from_utf8_lossy(&pk.topic);
                    if topic.len() < 9 {
                        return;
                    }

                    if topic.starts_with("heating/") {
                        if !topic.ends_with("/set") {
                            return;
                        }
                        
                        if topic.ends_with("/temp/set") {                                               // heating/{room_id}/temp/set
                            match u8::from_str_radix(&topic[8..(topic.len() - 9)], 10) {
                                Ok(room_id) => Self::set_temp(
                                    room_id, 
                                    String::from_utf8_lossy(&pk.payload).parse::<f32>().unwrap_or(5.0) as u8, 
                                    ch
                                ),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        } else if topic.ends_with("/power/set") {
                            match u8::from_str_radix(&topic[8..(topic.len() - 10)], 10) {               // heating/{room_id}/power/set
                                Ok(room_id) => Self::set_power(
                                    room_id, 
                                    String::from_utf8_lossy(&pk.payload).parse::<bool>().unwrap_or(false), 
                                    ch
                                ),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        } else if topic.ends_with("/mode/set") {
                            match u8::from_str_radix(&topic[8..(topic.len() - 9)], 10) {               // heating/{room_id}/mode/set
                                Ok(room_id) => Self::set_mode(
                                    room_id, 
                                    &String::from_utf8_lossy(&pk.payload), 
                                    ch
                                ),
                                Err(e) => eprintln!("{:?}", e),
                            }
                        }
                    }
        }
    }

    fn subscribe(&self, link_tx: &mut LinkTx) {
        link_tx.subscribe("heating/+/+/set").unwrap();
    }

    fn new() -> Box<dyn Thing<HyundaiPacket> + Send> where Self: Sized {
        Box::new(Self{ _marker: PhantomData})
    }
}