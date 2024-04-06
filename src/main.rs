use sysinfo::Networks;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::ipv6::Ipv6Packet;
use pnet::packet::Packet;
fn main() {
    //funny_print();
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
