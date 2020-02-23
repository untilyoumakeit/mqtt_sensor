extern crate eventual;
extern crate log;
use eventual::Timer;
use serde::{Serialize, Deserialize};

mod client;
mod probes;

use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::collections::BTreeMap;
use std::env;

use std::fs::File;
use std::io::prelude::*;

type ProbeMessage = Vec<probes::Probe>;

fn main() {

    let args: Vec<String> = env::args().collect();
    let default_path = "configuration.yaml".to_string();
    let config_path = args.last().unwrap_or(&default_path);
    let config = load_config(config_path);

    let (tx, rx) = channel();
    let (timer_tx, timer_rx) = channel();
    // Thread for probes
    thread::spawn(move || {
        susbscribe_probs(&tx, timer_rx);
    });

    // Client thread
    let mqtt = config.mqtt;
    thread::spawn(move || {
        client_listener(rx, &mqtt);
    });

    // Create a timer for probs
    let seconds = config.interval;
    let ticks = Timer::new().interval_ms(seconds * 1_000).iter();
    tick(&timer_tx); // First tick
    for _ in ticks {
        // timer_tx.send(()).unwrap();
        tick(&timer_tx);
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Configuration {
    mqtt: client::Configuration,
    interval: u32,
}

fn tick(tx: &Sender<()>) {
    match tx.send(()) {
        Ok(_) => { }
        Err(err) => println!("Can't send a tick message {}", err)
    }
}

fn load_config(path: &str) -> Configuration {
    // Load configuration
    println!("Load configuration from {}", path);
    let mut file = File::open(path).expect("Unable to open configuration file");
    let mut content = String::new();

    file.read_to_string(&mut content).expect("Unable to read configuration file");
    serde_yaml::from_str(&content).expect("Can't parse a configuration file")
}

fn susbscribe_probs(tx: &Sender<ProbeMessage>, rx: Receiver<()>) {
    loop {
        match rx.recv() { // Wait for a signal to work
            Ok(_) => {
                let vec = probes::get_probes();
                if let Err(e) = tx.send(vec) { // Do probes and send it back
                    println!("Can't send a vector of probes. {}", e.to_string()); // TODO use logs here
                }
            }
            Err(e) => println!("Can't process with probes. {}", e.to_string())
        }
        rx.recv().unwrap();
    }
}

impl client::Message for probes::Probe {
    fn write(&self, map: &mut BTreeMap<String, String>) {
        match self {
            probes::Probe::CPUNumber(number) => { 
                map.insert(String::from("cpu_number"), number.to_string()); 
            }
            probes::Probe::LoadAvg(one, five, fifteen) => {
                map.insert(String::from("load_one"), one.to_string());
                map.insert(String::from("load_five"), five.to_string());
                map.insert(String::from("load_fifteen"), fifteen.to_string());
            }
            probes::Probe::MemInfo(total, free, avail, buffers, cached) => {
                map.insert(String::from("mem_total"), total.to_string());
                map.insert(String::from("mem_free"), free.to_string());
                map.insert(String::from("mem_avail"), avail.to_string());
                map.insert(String::from("mem_buffers"), buffers.to_string());
                map.insert(String::from("mem_cached"), cached.to_string());
            }
            probes::Probe::DiskInfo(total, free) => {
                map.insert(String::from("disk_total"), total.to_string());
                map.insert(String::from("disk_free"), free.to_string());
            }
        }
    }
}

fn client_listener(rx: Receiver<ProbeMessage>, config: &client::Configuration) {
    let client = client::Client::new(config);
    println!("Created new client");
    loop {
        let _ = rx.recv().map(|value| {
            client.send(value);
        });
    }
}
