use sysinfo::Networks;

fn main() {
    // Read all interfaces
    let mut networks = Networks::new_with_refreshed_list();
    print_interfaces_total(&networks);
    std::thread::sleep(std::time::Duration::from_secs(10));

    networks.refresh();

    println!("\nAfter 10 seconds:");

    print_interfaces(&networks);
}

fn print_interfaces_total(networks: &Networks) {
    for (interface_name, data) in networks {
        println!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.total_received(),
            data.total_transmitted()
        );
    }
}

fn print_interfaces(networks: &Networks) {
    for (interface_name, data) in networks {
        println!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.received(),
            data.transmitted()
        );
    }
}
