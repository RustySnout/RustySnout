fn main() {
    use sysinfo::Networks;

    // Read all interfaces
    let mut networks = Networks::new_with_refreshed_list();
    for (interface_name, data) in &networks {
        println!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.total_received(),
            data.total_transmitted()
        );
    }
    std::thread::sleep(std::time::Duration::from_secs(10));

    networks.refresh();

    println!("\nAfter 10 seconds:");

    for(interface_name, data) in &networks {
        println!(
            "{interface_name}: {} B (down) / {} B (up)",
            data.received(),
            data.transmitted()
        )
    }
}
