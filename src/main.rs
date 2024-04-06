// use sysinfo::Networks;

// use pnet::datalink::Channel::Ethernet;
// use pnet::datalink::{self, NetworkInterface};
// use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
// use pnet::packet::ipv4::Ipv4Packet;
// use pnet::packet::ipv6::Ipv6Packet;
// use pnet::packet::Packet;

use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};
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
