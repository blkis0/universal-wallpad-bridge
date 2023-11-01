use core::panic;
use std::{sync::{mpsc::{Receiver, TryRecvError, self}, Arc, Mutex}, time::Duration, thread, io};

use rumqttd::local::LinkTx;

use crate::things::Channels;

use self::packet::{SerialPacket, PacketHandler, hyundai::HyundaiPacket, PacketParseError};

pub mod packet;
pub mod data;


pub struct Serial<T> where T: SerialPacket {
    _marker: std::marker::PhantomData<T>,
    
    pub path: String,
    pub baud_rate: u32,

    pub handlers: Vec<Box<dyn PacketHandler<T> + Send>>,
    pub channels: Channels<T>,

    pub rx: Receiver<T>,

    pub millis: Duration,
    pub print_variois: bool
}

pub trait ISerial<T: SerialPacket> {
    fn new(path: String, handlers: Vec<Box<dyn PacketHandler<T> + Send>>, link_tx: Arc<Mutex<LinkTx>>, delay: Duration, print_various: bool) -> Serial::<T> {
        let (tx, rx) = mpsc::channel();

        Serial::<T> {
            _marker: std::marker::PhantomData,
    
            path: path,
            baud_rate: T::baud_rate(),
    
            handlers: handlers,
            channels: Channels { link_tx: link_tx, serial_tx: tx },
            rx: rx,
            millis: delay,
            print_various
        }
    }

    fn path(&self) -> &String;
    fn baud_rate(&self) -> u32;
    fn handlers(&self) -> &Vec<Box<dyn PacketHandler<T> + Send>>;
    fn channels(&self) -> &Channels<T>;
    fn rx(&self) -> &Receiver<T>;
    fn various(&self) -> bool;

    fn prefix(&self) -> &'static [u8];
    fn suffix(&self) -> &'static [u8];
    fn millis(&self) -> &Duration;

    fn start(&self) {
        let i: u32 = 0;

        'open: loop {
            let builder = serialport::new(self.path(), self.baud_rate())
                .timeout(*self.millis());
 

            match builder.open() {
                Ok(mut port) => {
                    println!("Serial is opened on {}, baud: {}, attempt: {}", i, self.path(), self.baud_rate());
                    println!("{:?} {:?}", self.path(), port);

                    let mut buf: Vec<u8> = Vec::new();

                    let mut last_pkt: Option<T> = None;
                    let mut retry_delayed = false;

                    'clear: loop {
                        match self.rx().try_recv() {
                            Ok(_) => (),
                            Err(ref e) if *e == TryRecvError::Empty => break 'clear,
                            Err(e) => panic!("Serial channel(receiver) must be opened, {:?}", e)
                        }
                    } // Clear receiver pool

                    let mut tmp: [u8; 1024] = [0; 1024];

                    loop {
                        match port.read(&mut tmp) {
                            Ok(l) => {
                                let ref_tmp = &tmp[0..l];      
                                if self.various() {
                                    println!("{:?} {:?}", self.path(), ref_tmp);
                                }

                                if !buf.is_empty() {
                                    if ref_tmp.ends_with(self.suffix()) {
                                        buf.extend_from_slice(&ref_tmp);
                                        
                                        retry_delayed = !self._handle(&buf, &last_pkt);

                                        if !retry_delayed {
                                            last_pkt = None;
                                        }

                                        buf.clear();
                                        
                                    } else {
                                        buf.extend_from_slice(&ref_tmp);
                                    }
                                } else if ref_tmp.starts_with(self.prefix()) {
                                    if ref_tmp.ends_with(self.suffix()) {
                                        retry_delayed = !self._handle(&ref_tmp[0..ref_tmp.get(1).map(|v| *v as usize).unwrap_or(l).min(l)], &last_pkt);

                                        if !retry_delayed {
                                            last_pkt = None;
                                        }
                                    } else {
                                        buf.clear();
                                        buf.extend_from_slice(&ref_tmp);
                                    }
                                } else {
                                    if last_pkt.is_some() {
                                        eprintln!("Error detected, retry send -> {:?}", &last_pkt);
                                        retry_delayed = true;
                                    }
                                }

                                let t = T::length_from_buffer(&buf);

                                if t.is_some() && buf.len() >= t.unwrap() {

                                    retry_delayed = !self._handle(&buf[0..t.unwrap().min(buf.len())], &last_pkt);

                                    if !retry_delayed {
                                        last_pkt = None;
                                    }
     
                                    if buf[t.unwrap()..buf.len()].starts_with(self.prefix()) {
                                        buf = buf.split_off(t.unwrap())
                                    } else {
                                        buf.clear();
                                    }
                                }
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => {
                                if retry_delayed {
                                    match port.write(&last_pkt.as_ref().unwrap().to_vec()) {
                                        Ok(c) => {
                                            println!("retried -> {:?}", c);
                                            retry_delayed = false;
                                        },
                                        Err(e) => eprintln!("{:?}", e)
                                    };
                                } else {

                                match self.rx().try_recv() {
                                    Ok(v) => {
                                        match port.write(&v.to_vec()) {
                                            Ok(_) => {
                                                println!("-> {:?}", v)
                                            },
                                            Err(e) => eprintln!("{:?}", e)
                                        }
                                        
                                        /*if c > 0 {
                                            match utils::serial_tx_lock(&self.channels().serial_tx) {
                                                Some(serial_tx) => {let _ = serial_tx.send((v, c - 1));},
                                                _ => ()
                                            }
                                        } else {
                                            */
                                            last_pkt = Some(v);
                                        //}
                                        
                                    },
                                    Err(ref e) if *e == TryRecvError::Empty => (),
                                    Err(e) => panic!("Serial channel(receiver) must be opened, {:?}", e)
                                }
                            }
                            },
                            Err(ref e) if e.kind() == io::ErrorKind::BrokenPipe => continue 'open,
                            Err(e) => eprintln!("{:?}", e),
                        }
                        
                        
                    }
                },
                Err(e) => eprintln!("{:?}", e),
            }

            thread::sleep(Duration::from_secs(5));
        }
    }
    fn _handle(&self, buf: &[u8], last_pkt: &Option<T>) -> bool {
        match self.handle(&buf) {
            Ok(packet) => {
                'chain: for handler in self.handlers() {
                    if handler.handle(&packet, &self.channels()) && !handler.chaining() {
                        break 'chain;
                    }
                }

                if last_pkt.is_some() && !last_pkt.as_ref().unwrap().is_correct_response(&packet) {
                    eprintln!("Response drop, retry send -> {:?}", &last_pkt);
                    return false;
                }

                return true;
            },
            Err(e) => {
                eprintln!("Buffer {:?} {:?}", e, &buf);
                if last_pkt.is_some() {
                    eprintln!("Error detected, retry send -> {:?}", &last_pkt);
                    return false;
                }
                return true;
            }
        }
    }

    fn handle(&self, buf: &[u8]) -> Result<T, PacketParseError>;
}

impl ISerial<HyundaiPacket> for Serial<HyundaiPacket> {
    fn path(&self) -> &String { &self.path }
    fn baud_rate(&self) -> u32 { self.baud_rate }
    fn handlers(&self) -> &Vec<Box<dyn PacketHandler<HyundaiPacket> + Send>> { &self.handlers }
    fn channels(&self) -> &Channels<HyundaiPacket> { &self.channels }
    fn rx(&self) -> &Receiver<HyundaiPacket> { &self.rx }

    fn prefix(&self) -> &'static [u8] { HyundaiPacket::PREFIX }
    fn suffix(&self) -> &'static [u8] { HyundaiPacket::SUFFIX }

    fn handle(&self, buf: &[u8]) -> Result<HyundaiPacket, PacketParseError> {
        return HyundaiPacket::parse(buf);
    }

    fn millis(&self) -> &Duration {
        &self.millis
    }

    fn various(&self) -> bool { self.print_variois }
}

