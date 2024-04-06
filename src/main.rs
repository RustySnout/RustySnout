// use sysinfo::Networks;

// use pnet::datalink::Channel::Ethernet;
// use pnet::datalink::{self, NetworkInterface};
// use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
// use pnet::packet::ipv4::Ipv4Packet;
// use pnet::packet::ipv6::Ipv6Packet;
// use pnet::packet::Packet;

use regex::Regex;

use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};

struct Process {
    id: String,
    name: String,
    up_bps: u64,
    down_bps: u64,
    connections: u32,
}

struct Connection {
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

fn main() -> io::Result<()> {
    //funny_print();
    //listen_for_packets();

    // sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep /path/to/bandwhich
    let mut child = Command::new("bandwhich")
        .arg("--raw")
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.stdout.take().expect("Failed to open stdout");
    let reader = BufReader::new(output);

    let mut refresh_buffer = String::new();

    // Process each line of the output in a loop
    for line in reader.lines() {
        let line = line?;
        // Detect the start of a new refresh block
        if line.starts_with("Refreshing:") {
            // Process the previous refresh block, if any
            if !refresh_buffer.is_empty() {
                process_refresh_buffer(&refresh_buffer)?;
                refresh_buffer.clear();
            }
            continue;
        }
        // Add the line to the current refresh block
        refresh_buffer.push_str(&line);
        refresh_buffer.push('\n');
    }

    // Process the last refresh block, if any
    if !refresh_buffer.is_empty() {
        process_refresh_buffer(&refresh_buffer)?;
    }

    Ok(())
}

fn process_refresh_buffer(refresh_buffer: &str) -> io::Result<()> {
    // Process the refresh block here
    println!("Refresh block:\n{}", refresh_buffer);
    Ok(())
}

fn parse_sample_output(sample_output: &str) {
    let sample_output = r#"
    Refreshing:
    process: <1712433282> "gns3server" up/down Bps: 304/0 connections: 2
    process: <1712433282> "spotify" up/down Bps: 47/59 connections: 5
    process: <1712433282> "msedge" up/down Bps: 0/50 connections: 1
    connection: <1712433282> <lo>:8080 => localhost.:46418 (tcp) up/down Bps: 240/0 process: "gns3server"
    connection: <1712433282> <lo>:46418 => localhost.:8080 (tcp) up/down Bps: 64/0 process: "gns3server"
    connection: <1712433282> <wlp3s0>:5353 => 192.168.43.1:5353 (udp) up/down Bps: 0/50 process: "msedge"
    connection: <1712433282> <wlp3s0>:5353 => fe80::6c18:d9ff:fec0:4782:5353 (udp) up/down Bps: 0/50 process: "spotify"
    connection: <1712433282> <wlp3s0>:41815 => 239.255.255.250:1900 (udp) up/down Bps: 26/0 process: "spotify"
    connection: <1712433282> <wlp3s0>:5353 => ff02::fb:5353 (udp) up/down Bps: 10/0 process: "spotify"
    connection: <1712433282> <wlp3s0>:5353 => mdns.mcast.net.:5353 (udp) up/down Bps: 10/0 process: "spotify"
    connection: <1712433282> <wlp3s0>:55002 => 124.65.199.104.bc.googleusercontent.com.:4070 (tcp) up/down Bps: 0/8 process: "spotify"
    remote_address: <1712433282> localhost. up/down Bps: 304/0 connections: 2
    remote_address: <1712433282> 192.168.43.1 up/down Bps: 0/50 connections: 1
    remote_address: <1712433282> fe80::6c18:d9ff:fec0:4782 up/down Bps: 0/50 connections: 1
    remote_address: <1712433282> 239.255.255.250 up/down Bps: 26/0 connections: 1
    remote_address: <1712433282> ff02::fb up/down Bps: 10/0 connections: 1
    remote_address: <1712433282> mdns.mcast.net. up/down Bps: 10/0 connections: 1
    remote_address: <1712433282> 124.65.199.104.bc.googleusercontent.com. up/down Bps: 0/8 connections: 1
"#; 


    let process_re = Regex::new(r#"process: <(\d+)> "([^"]+)" up/down Bps: (\d+)/(\d+) connections: (\d+)"#).unwrap();
    let connection_re = Regex::new(r#"connection: <(\d+)> <([^>]+)>:([^ ]+) => ([^:]+):(\d+) \(([^)]+)\) up/down Bps: (\d+)/(\d+) process: "([^"]+)""#).unwrap();
    let remote_address_re = Regex::new(r#"remote_address: <(\d+)> ([^ ]+) up/down Bps: (\d+)/(\d+) connections: (\d+)"#).unwrap();

    let mut processes: Vec<Process> = Vec::new();
    let mut connections: Vec<Connection> = Vec::new();
    let mut remote_addresses: Vec<RemoteAddress> = Vec::new();

    for line in sample_output.lines() {
        if let Some(caps) = process_re.captures(line) {
            let process = Process {
                id: caps[1].to_string(),
                name: caps[2].to_string(),
                up_bps: caps[3].parse::<u64>().unwrap(),
                down_bps: caps[4].parse::<u64>().unwrap(),
                connections: caps[5].parse::<u32>().unwrap(),
            };
            processes.push(process);
        } else if let Some(caps) = connection_re.captures(line) {
            let connection = Connection {
                id: caps[1].to_string(),
                source: caps[2].to_string(),
                destination: caps[4].to_string(), 
                protocol: caps[6].to_string(), 
                up_bps: caps[7].parse::<u64>().unwrap(),
                down_bps: caps[8].parse::<u64>().unwrap(), 
                process: caps[9].to_string(), 
            };
            connections.push(connection);
        } else if let Some(caps) = remote_address_re.captures(line) {
            let remote_address = RemoteAddress {
                id: caps[1].to_string(),
                address: caps[2].to_string(),
                up_bps: caps[3].parse::<u64>().unwrap(),
                down_bps: caps[4].parse::<u64>().unwrap(),
                connections: caps[5].parse::<u32>().unwrap(),
            };
            remote_addresses.push(remote_address);
        }
    }

    println!("\nProcesses:");
    for process in processes {
        println!("ID: {}, Name: {}, Up/Down Bps: {}/{}, Connections: {}", 
            process.id, process.name, process.up_bps, process.down_bps, process.connections);
    }
    
    println!("\nConnections:");
    for connection in connections {
        println!("ID: {}, Source: {}, Destination: {}, Protocol: {}, Up/Down Bps: {}/{}, Process: {}", 
            connection.id, connection.source, connection.destination, connection.protocol, connection.up_bps, connection.down_bps, connection.process);
    }

    println!("\nRemote Addresses:");
    for remote_address in remote_addresses {
        println!("ID: {}, Address: {}, Up/Down Bps: {}/{}, Connections: {}", 
            remote_address.id, remote_address.address, remote_address.up_bps, remote_address.down_bps, remote_address.connections);
    }

   
}

/*fn listen_for_packets() {
    let interfaces = datalink::interfaces();
    // Allow user to select interface
    for (i, interface) in interfaces.iter().enumerate() {
        println!("{}: {:?}", i, interface);
    }
    println!("Please select an interface to capture packets: ");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let index = input.trim().parse::<usize>().unwrap();

    let interface = interfaces[index].clone();

    let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
        Ok(Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Unhandled channel type"),
        Err(e) => panic!("An error occurred: {}", e),
    };
    let mut count = 0;
    loop {
        println!("\nPacket number: {}", count);
        count += 1;
        match rx.next() {
            Ok(packet) => {
                if let Some(eth) = EthernetPacket::new(packet) {
                    println!("Ethernet type: {:?}", eth.get_ethertype());

                    println!("Source MAC: {:?}", eth.get_source());
                    println!("Destination MAC: {:?}", eth.get_destination());

                    match eth.get_ethertype() {
                        EtherTypes::Ipv4 => {
                            if let Some(ipv4) = Ipv4Packet::new(eth.payload()) {
                                println!("IPv4 source: {:?}", ipv4.get_source());
                                println!("IPv4 destination: {:?}", ipv4.get_destination());
                                println!("IPv4 payload: {:?}", ipv4.payload());
                            }
                        }
                        EtherTypes::Ipv6 => {
                            if let Some(ipv6) = Ipv6Packet::new(eth.payload()) {
                                println!("IPv6 source: {:?}", ipv6.get_source());
                                println!("IPv6 destination: {:?}", ipv6.get_destination());
                                println!("IPv6 payload: {:?}", ipv6.payload());
                            }
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Error receiving packet: {}", e);
            }
        }
    }
}
fn funny_print() {
    let mut networks = Networks::new_with_refreshed_list();
    println!("Total information for all interfaces:");
    print_interfaces_total(&networks);
    println!("\nAfter 10 seconds:");
    print_interfaces_after_x(&mut networks, 10);
}

fn print_interfaces_total(networks: &Networks) {
    for (interface_name, data) in networks {
        // print interface name and total data received and transmitted and total packets received and transmitted
        println!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.total_received(),
            data.total_transmitted()
        );
        println!(
            "{interface_name}: {} packets (down) / {} packets (up)",
            data.total_packets_received(),
            data.total_packets_transmitted()
        );
    }
}

fn print_interfaces(networks: &Networks) {
    for (interface_name, data) in networks {
        // print interface name and data received and transmitted and packets received and transmitted
        println!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.received(),
            data.transmitted()
        );
        println!(
            "{interface_name}: {} packets (down) / {} packets (up)",
            data.packets_received(),
            data.packets_transmitted()
        );
    }
}

fn print_interfaces_after_x(networks: &mut Networks, x: u64) {
    std::thread::sleep(std::time::Duration::from_secs(x));

    networks.refresh();

    print_interfaces(networks);
}
*/
