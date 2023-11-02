use std::marker::PhantomData;

use rumqttd::{local::LinkTx, protocol::Publish};

use crate::{serial::{packet::{hyundai::{HyundaiPacket, HyundaiPacketHandler}, SerialPacket, Command, PacketHandler}, data::{realtime_energy_packet::RealtimeEnergyDataPacket, Data}}, utils};

use super::{Thing, Channels};

#[derive(Clone)]
pub struct RealtimeEnergyMeter<T: SerialPacket> {
    _marker: PhantomData<T>,
}

//const REQUEST_PACKET: &[u8] = &[0xF7, 0x0B, 0x01, 0x43, 0x01, 0x1F, 0x11, 0x00, 0x00, 0xB1, 0xEE];

impl RealtimeEnergyMeter<HyundaiPacket> {
    fn on_response(packet: &HyundaiPacket, channels: &Channels<HyundaiPacket>) {
        match RealtimeEnergyDataPacket::parse::<HyundaiPacket>(&packet.data) {
            Ok(data) => {
                println!("{:?}", data);
                
                match utils::link_tx_lock(&channels.link_tx.clone()) {
                    Some(mut link_tx) => {
                        let mut result = Ok(0);

                        if data.electric.is_some() {
                            result = link_tx.publish("electric/meter", data.electric.unwrap().to_string());
                            
                        }

                        if data.water.is_some() {
                            result = result.and(link_tx.publish("water/meter", data.water.unwrap().to_string()));
                        }

                        if data.gas.is_some() {
                            result = result.and(link_tx.publish("gas/meter", data.gas.unwrap().to_string()));
                        }

                        if result.is_err() {
                            eprintln!("{:?}", result.unwrap_err());
                        }
                    }
                    _ => ()
                }
            },
            Err(e) => eprintln!("{:?}", e)
        }
    }
}


impl Thing<HyundaiPacket> for RealtimeEnergyMeter<HyundaiPacket> {
    fn handler(&self) -> Box<dyn PacketHandler<HyundaiPacket> + Send> {
        Box::new(HyundaiPacketHandler {
            device_id: Some(0x43),
            device_sub_id: Some(0x1F),
            room_id: Some(0x11),
            callback: Box::new(|pk, ch| { 
                match pk.command {
                    Command::Response => {
                        Self::on_response(pk, ch);
                    },
                    _ => ()
                }
            }),
            chaining: false,
            is_primary: false
        })
    }

    fn task(&self) -> Option<fn(&Channels<HyundaiPacket>, &Option<&Channels<HyundaiPacket>>)> {
        Some(|_ch_1, ch_2| {
        if ch_2.is_none() {
            return;
        }

        //match ch_2.unwrap().serial_tx.lock() {
        //    Ok(port) => {
                let r = ch_2.unwrap().serial_tx.send(HyundaiPacket::new(0x43, Command::Request, 0x1F, 0x11, RealtimeEnergyDataPacket::create_request::<HyundaiPacket>().unwrap()));
                if r.is_err() {
                    eprintln!("Realtime Energy Meter {:?}", r.err());
                }
            }
        )

       

    }

    fn topic_handler(&self) -> fn(&Publish, &Channels<HyundaiPacket>, &Option<&Channels<HyundaiPacket>>) {
        |_, _, _| {}
    }

    fn subscribe(&self, _link_tx: &mut LinkTx) {
        
    }

    fn new() -> Box<dyn Thing<HyundaiPacket> + Send> where Self: Sized {
        Box::new(Self { _marker: PhantomData, })
    }
}
