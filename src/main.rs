
use clap::Parser;
use rumqttd::local::{LinkRx, LinkTx};
use rumqttd::{Broker, Config, Notification};

use universal_wallpad_bridge::serial::packet::{PacketHandler, SerialPacket};
use universal_wallpad_bridge::serial::packet::hyundai::HyundaiPacket;
use universal_wallpad_bridge::serial::{Serial, packet::Manufacturer, ISerial};
use universal_wallpad_bridge::things::{Feature, Channels};

mod cli;

use std::{thread, sync::{Arc, Mutex}, time::Duration};

fn main() {
    let args = cli::Args::parse();

    if args.various {
        tracing_subscriber::fmt()
            .pretty()
            .with_line_number(false)
            .with_file(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .try_init()
            .expect("initialized subscriber succesfully");
    }

    let (mut broker, mut link_rx, mut link_tx) = create_broker(&args.rumqttd, args.various);

    let mut topic_handlers = Vec::new();
    let mut tasks = Vec::new();

    let mut pri_pkt_handlers = Vec::new();
    let mut sec_pkt_handlers = Vec::new();

    match args.manufacturer {
        Manufacturer::HyundaiHT => {
            if !args.features.is_empty() {

                for f in &args.features {
                    let t = f.new::<HyundaiPacket>();

                    let h = t.handler();
                    if h.is_primary() {
                        pri_pkt_handlers.push(h);
                    } else {
                        sec_pkt_handlers.push(h);
                    }
                    
                    topic_handlers.push(t.topic_handler());

                    let tk = t.task();
                    if tk.is_some() {
                        tasks.push(tk.unwrap());
                    }

                    if !args.various {
                        t.subscribe(&mut link_tx)
                    }

                }
            } else {
                let things = Feature::defaults::<HyundaiPacket>();
                
                for t in things {
                    let h = t.handler();
                    if h.is_primary() {
                        pri_pkt_handlers.push(h);
                    } else {
                        sec_pkt_handlers.push(h);
                    }

                    topic_handlers.push(t.topic_handler());

                    let tk = t.task();
                    if tk.is_some() {
                        tasks.push(tk.unwrap());
                    }
                }
            }
        }
    }

    thread::spawn(move || {
        broker.start().unwrap();
    });

    let a_link_tx = Arc::new(Mutex::new(link_tx));

    let (pri_serial, pri_channel) = match args.manufacturer {
        Manufacturer::HyundaiHT => create_serial::<HyundaiPacket>(
            args.primary_port, 
            pri_pkt_handlers, 
            a_link_tx.clone(),
            Duration::from_millis(10)
        )
    };

    let (sec_serial, sec_channel) = if args.second_port.is_some() {
        match args.manufacturer {
            Manufacturer::HyundaiHT => {
                let s = create_serial::<HyundaiPacket>(
                    args.second_port.unwrap(), 
                    sec_pkt_handlers,
                    a_link_tx,
                    Duration::from_millis(800)
                );

                (Some(s.0), Some(s.1))
            }
        }
    } else {
        (None, None)
    };

    thread::spawn(move || { // Primary Serial
        pri_serial.start();
    });

    if sec_serial.is_some() {
        thread::spawn(move || { // Secondary Serial
            sec_serial.unwrap().start();
        });
    }
    
    let pri_channel_t = pri_channel.clone();
    let sec_channel_t = sec_channel.clone();


    thread::spawn(move || { // Task Loop
        loop {
            thread::sleep(Duration::from_secs(args.interval));

            for task in &tasks {
                task(&pri_channel_t, &sec_channel_t.as_ref());
            }
        }
    });

    loop { // MQTT Broker
        let notification = match link_rx.recv().unwrap() {
            Some(v) => v,
            None => continue,
        };

        match notification {
            Notification::Forward(forward) => {
                for topic_handler in &topic_handlers { // Handling a topic
                    (topic_handler)(&forward.publish, &pri_channel, &sec_channel.as_ref());
                }
                
                println!(
                    "Topic = {:?}, Retain = {:?}, Payload = {} bytes",
                    forward.publish.topic,
                    forward.publish.retain,
                    forward.publish.payload.len()
                );
            }
            v => {
                println!("{v:?}");
            }
        }
    }   
}

fn create_broker(config_path: &str, various: bool) -> (Broker, LinkRx, LinkTx) {
    if various {
        tracing_subscriber::fmt()
            .pretty()
            .with_line_number(false)
            .with_file(false)
            .with_thread_ids(false)
            .with_thread_names(false)
            .try_init()
            .expect("initialized subscriber succesfully");
    }

    let mqtt_config = config::Config::builder()
        .add_source(config::File::with_name(config_path))
        .build()
        .unwrap();

    let mqtt_config: Config = mqtt_config.try_deserialize().unwrap();

    if various {
        dbg!(&mqtt_config);
    }

    let broker = Broker::new(mqtt_config);
    let (link_tx, link_rx) = broker.link("universal-wallpad-bridge").unwrap();

    (broker, link_rx, link_tx)
}

fn create_serial<T: SerialPacket + Sized>(port_path: String, pkt_handlers: Vec<Box<dyn PacketHandler<T> + Send>>, link_tx: Arc<Mutex<LinkTx>>, delay: Duration) -> (Serial<T>, Channels<T>) where Serial<T>: ISerial<T> {
    let serial: Serial<T> = Serial::<T>::new(
        port_path, 
        pkt_handlers,
        link_tx,
        delay
    );

    let c = serial.channels.clone();

    (serial, c)
}