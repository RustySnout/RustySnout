use sysinfo::Networks;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, NetworkInterface};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::Packet;
fn main() {
    // Read all interfaces
    let mut networks = Networks::new_with_refreshed_list();
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
