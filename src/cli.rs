use clap::Parser;

use universal_wallpad_bridge::serial::packet::Manufacturer;
use universal_wallpad_bridge::things::Feature;

#[derive(Parser)] // requires `derive` feature
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    
    /// Select your wall pad manufacturer
    #[clap(short = 'm', long)]
    pub manufacturer: Manufacturer,
    
    /// Select the available devices (Separated by spaces)
    #[clap(short = 'f', long, value_delimiter = ',')]
    pub features: Vec<Feature>,

    /// A serial port connected to the door lock and energy meter (ex. COM2 or /dev/ttyUSB1, etc...)
    #[clap(short = 's', long, value_name = "PATH")]
    pub second_port: Option<String>,

    /// Specified path for rumqttd configuration
    #[clap(short = 'r', long, value_name = "PATH", default_value_t = ("./rumqttd.toml".to_string()))]
    pub rumqttd: String,

    /// Fetch Interval
    #[clap(short = 'i', long, default_value_t = 2)]
    pub interval: u64,

    /// Enable log
    #[clap(short = 'l', long, value_name = "PATH")]
    pub log: Option<String>,

    /// Print more various information
    #[clap(short = 'v', default_value_t = false)]
    pub various: bool,

    /// A serial port connected to the entire device (ex. COM1 or /dev/ttyUSB0, etc...)
    #[clap(last = true, value_name = "PATH")]
    pub primary_port: String,
}