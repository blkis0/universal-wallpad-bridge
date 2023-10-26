use std::{sync::{Mutex, Arc, MutexGuard}, time::Duration};

use rumqttd::local::LinkTx;

pub fn xor_checksum(buf: &[u8], len: usize) -> u8 {
    let mut result: u8 = 0x00;

    for i in 0..len {
        result ^= buf[i];
    }

    return result;
}

pub fn link_tx_lock(link_tx: &Arc<Mutex<LinkTx>>) -> Option<MutexGuard<'_, LinkTx>> {
    match link_tx.lock() {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("{:?}", e);
            return None;
        },
    }
}

pub trait DurationUtils {
    fn from_minutes(value: u64) -> Duration;
    fn as_minutes(&self) -> u64;
    fn as_section_minutes(&self) -> u64;
    fn as_hours(&self) -> u64;
}

impl DurationUtils for Duration {
    fn from_minutes(value: u64) -> Duration {
        Duration::from_secs(value * 60)
    }

    fn as_minutes(&self) -> u64 {
        self.as_secs() / 60
    }

    fn as_section_minutes(&self) -> u64 {
        self.as_secs() % 3600 / 60
    }

    fn as_hours(&self) -> u64 {
        self.as_secs() / 3600
    }
}

