// sudo apt update
// sudo apt install libsqlite3-dev
// cargo install bandwhich
// sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep /path/to/bandwhich

// use sysinfo::Networks;
mod cli;
mod display;
mod network;
mod os;

use crate::os::ProcessInfo;

use network::{
    dns::{self, IpTable},
    LocalSocket, Sniffer, Utilization,
};

use std::{
    collections::HashMap,
    fmt,
    fs::File,
    net::Ipv4Addr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    thread::{self, park_timeout},
    time::{Duration, Instant},
};

use anyhow::Ok;
// use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self /*NetworkInterface*/};
// use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
// use pnet::packet::ipv4::Ipv4Packet;
// use pnet::packet::ipv6::Ipv6Packet;
// use pnet::packet::Packet;

//use pnet::packet::ip;
use regex::Regex;

//use core::time;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    terminal,
};

use rusqlite::{params, Connection /* , Result*/};

use chrono::{/*DateTime,*/ Utc};
use pnet::datalink::{DataLinkReceiver, NetworkInterface};

use crate::cli::Opt;

struct Process {
    id: String,
    name: String,
    up_bps: u64,
    down_bps: u64,
    connections: u32,
}

struct NetwrokConnection {
    id: String,
    source: String,
    destination: String,
    protocol: String,
    up_bps: u64,
    down_bps: u64,
    process: String,
}

struct RemoteAddress {
    id: String,
    address: String,
    up_bps: u64,
    down_bps: u64,
    connections: u32,
}

pub struct OpenSockets {
    sockets_to_procs: HashMap<LocalSocket, ProcessInfo>,
}

pub struct OsInputOutput {
    pub interfaces_with_frames: Vec<(NetworkInterface, Box<dyn DataLinkReceiver>)>,
    pub get_open_sockets: fn() -> OpenSockets,
    pub terminal_events: Box<dyn Iterator<Item = Event> + Send>,
    pub dns_client: Option<dns::Client>,
    pub write_to_stdout: Box<dyn FnMut(String) + Send>,
}

fn main() -> anyhow::Result<()> {
    // print all interface names and allow user to select it as Option<&str>
    let interfaces = datalink::interfaces();
    for (i, interface) in interfaces.iter().enumerate() {
        println!("{}: {:?}", i, interface);
    }
    println!("Please select an interface to capture packets: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let index = input.trim().parse::<usize>().unwrap();
    let interface_name = &interfaces[index].name;

    // make interface_name as Option<&str>
    let interface_name: Option<&str> = Some(&interface_name);

    // set ip to 127.0.0.53 as Option<Ipv4Addr>
    let dns_server: Option<Ipv4Addr> = Some(Ipv4Addr::new(127, 0, 0, 53));

    // create Opt struct with interface_name and dns_server
    let os_input = os::get_input(interface_name, true, dns_server)?;

    let running = Arc::new(AtomicBool::new(true));
    let paused = Arc::new(AtomicBool::new(false));
    let last_start_time = Arc::new(RwLock::new(Instant::now()));
    let cumulative_time = Arc::new(RwLock::new(Duration::new(0, 0)));
    let ui_offset = Arc::new(AtomicUsize::new(0));
    let dns_shown = true;

    let terminal_events = os_input.terminal_events;
    let get_open_sockets = os_input.get_open_sockets;
    let mut write_to_stdout = os_input.write_to_stdout;
    let mut dns_client = os_input.dns_client;

    let network_utilization = Arc::new(Mutex::new(Utilization::new()));

    let mut active_threads: Vec<_> = vec![];
    let network_utilization = Arc::new(Mutex::new(Utilization::new()));

    let sniffer_threads = os_input
        .interfaces_with_frames
        .into_iter()
        .map(|(iface, frames)| {
            let name = format!("sniffing_handler_{}", iface.name);
            let running = running.clone();
            let show_dns = true;
            let network_utilization = network_utilization.clone();

            thread::Builder::new()
                .name(name)
                .spawn(move || {
                    let mut sniffer = Sniffer::new(iface, frames, show_dns);

                    while running.load(Ordering::Acquire) {
                        if let Some(segment) = sniffer.next() {
                            network_utilization.lock().unwrap().update(segment);
                        }
                    }
                })
                .unwrap()
        })
        .collect::<Vec<_>>();
    active_threads.extend(sniffer_threads);

    for thread_handler in active_threads {
        thread_handler.join().unwrap()
    }
    Ok(())
}
