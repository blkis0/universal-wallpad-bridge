use rumqttd::{local::LinkTx, protocol::Publish};

use crate::{serial::{packet::{hyundai::{HyundaiPacket, HyundaiPacketHandler}, SerialPacket, Command, PacketHandler}, data::{Data, binary_switch_packet::BinarySwitchDataPacket}}, utils};

use super::{Thing, Channels};

#[derive(Clone)]
pub struct LivingRoomLight<T: SerialPacket> {
    _marker: std::marker::PhantomData<T>,
}

impl LivingRoomLight<HyundaiPacket> {
    fn on_response(id: u8, buf: &[u8], channels: &Channels<HyundaiPacket>) {
        match BinarySwitchDataPacket::parse::<HyundaiPacket>(buf) {
            Ok(data) => {
                println!("{:?}", data);

                if data.status.is_some() {
                    match utils::link_tx_lock(&channels.link_tx.clone()) {
                        Some(mut link_tx) => {
                            let result = link_tx.publish(format!("light/0/{:0>2}", id), data.status.unwrap().to_string());    
    
                            if result.is_err() {
                                eprintln!("{:?}", result.unwrap_err());
                            }
                        }
                        _ => ()
                    }
                }
        
            },
            Err(e) => eprintln!("{:?}", e)
        }
    }

    fn set_status(room_id: u8, status: bool, channels: &Channels<HyundaiPacket>) {

                let p = HyundaiPacket::new(
                    0x19, Command::Modify, 0x40, 0x10 + room_id,
                    BinarySwitchDataPacket::create_modify::<HyundaiPacket>(status).unwrap()
                );

                match channels.serial_tx.send(p) {
                    Ok(_) => (),
                    Err(e) => eprintln!("{:?}", e)
                }

    }
}


impl Thing<HyundaiPacket> for LivingRoomLight<HyundaiPacket> {
    fn handler(&self) -> Box<dyn PacketHandler<HyundaiPacket> + Send> {
        Box::new(HyundaiPacketHandler {
            device_id: Some(0x19),
            device_sub_id: Some(0x40),
            room_id: None,
            callback: Box::new(|pk, ch| { 
                match pk.command {
                    Command::Response => {
                        match pk.room_id {
                            0x10 => {
                                Self::on_response(1, &pk.data[1..2], ch);
                                Self::on_response(2, &pk.data[2..3], ch);
                            },
                            0x11 => Self::on_response(1, &pk.data, ch),
                            0x12 => Self::on_response(2, &pk.data, ch),
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

            if topic.len() >= 12 {
                if topic.starts_with("light/") {
                    if !topic.ends_with("/set") {
                        return;
                    }

                    let id = &topic[6..(topic.len() - 4)];
                    let value = String::from_utf8_lossy(&pk.payload).parse::<bool>().unwrap_or(false);

                    match id {
                        "0/00" => {
                            Self::set_status(1, value, &ch);
                            Self::set_status(2, value, &ch);

                        },
                        "0/01" => {
                            Self::set_status(1, value, &ch);
                        },
                        "0/02" => {
                            Self::set_status(2, value, &ch)
                        },
                        _ => ()
                    }
                }
            }
        }
    }

    fn subscribe(&self, link_tx: &mut LinkTx) {
        link_tx.subscribe("light/0/+/set").unwrap();
    }

    fn new() -> Box<dyn Thing<HyundaiPacket> + Send> {
        Box::new(Self{ _marker: std::marker::PhantomData })
    }
}
