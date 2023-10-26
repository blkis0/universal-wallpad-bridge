use std::sync::{Arc, Mutex, mpsc::Sender};

use rumqttd::{local::LinkTx, protocol::Publish};

use crate::serial::packet::{PacketHandler, SerialPacket, Manufacturer};

use self::{floor_heating::FloorHeating, living_room_light::LivingRoomLight, realtime_energy_meter::RealtimeEnergyMeter, ventilator::Ventilator};

pub mod realtime_energy_meter;
pub mod floor_heating;
pub mod living_room_light;
pub mod ventilator;

pub trait Thing<T: SerialPacket> {
    fn handler(&self) -> Box<dyn PacketHandler<T> + Send>;
    fn topic_handler(&self) -> fn(&Publish, &Channels<T>, &Option<&Channels<T>>);
    fn subscribe(&self, link_tx: &mut LinkTx);

    fn task(&self) -> Option<fn(&Channels<T>, &Option<&Channels<T>>)>;
    fn new() -> Box<dyn Thing<T> + Send> where Self: Sized;
}

#[derive(Clone)]
pub struct Channels<T: SerialPacket> {
    pub link_tx: Arc<Mutex<LinkTx>>,
    pub serial_tx: Sender<T>
}

impl<T: SerialPacket> Clone for Box<dyn Thing<T> + Send> where Self: Thing<T> {
    fn clone(&self) -> Box<dyn Thing<T> + Send + 'static> {
        Self::new()
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone()
    }
}

#[derive(Clone)]
pub struct TopicHandler<T: SerialPacket> {
    pub handle: fn(&Publish, &Channels<T>)
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Feature {
    #[clap(name = "floor_heating")]
    FloorHeating,
    #[clap(name = "ventilator")]
    Ventilator,
    #[clap(name = "living_room_lights")]
    LivingRoomLights,
    #[clap(name = "realtime_energy_meter")]
    RealtimeEnergyMeter,
}

impl Feature {
    pub fn new<T: SerialPacket + 'static>(&self) -> Box<dyn Thing<T> + Send> 
        where FloorHeating<T>: Thing<T>,
              LivingRoomLight<T>: Thing<T>,
              RealtimeEnergyMeter<T>: Thing<T>,
              Ventilator<T>: Thing<T>,
    {
        match self {
            Feature::FloorHeating => FloorHeating::<T>::new(),
            Feature::Ventilator => Ventilator::<T>::new(),
            Feature::LivingRoomLights => LivingRoomLight::<T>::new(),
            Feature::RealtimeEnergyMeter => RealtimeEnergyMeter::<T>::new(),
        }
    }

    pub fn defaults<T: SerialPacket>() -> Vec<Box<dyn Thing<T> + Send>> 
        where FloorHeating<T>: Thing<T>,
              LivingRoomLight<T>: Thing<T>,
              RealtimeEnergyMeter<T>: Thing<T>,
              Ventilator<T>: Thing<T>,
    {
        match T::manufacturer() {
            Manufacturer::HyundaiHT => vec![
                FloorHeating::<T>::new(),
                Ventilator::<T>::new(),
                LivingRoomLight::<T>::new(),
                RealtimeEnergyMeter::<T>::new(),
            ],
        }
    }
}