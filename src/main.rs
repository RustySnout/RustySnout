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

use rusqlite::{params, Connection, Result};

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

struct AppRow {
    process_name: String,
}

struct ConnectionRow {
    cid: u32,
    source: String,
    destination: String,
    protocol: String,
    up_bps: u64,
    down_bps: u64,
    process_name: String,
}

struct RemoteAddressRow {
    rid: u32,
    address: String,
    up_bps: u64,
    down_bps: u64,
    connections: u32,
}

struct ProcessRow {
    pid: u32,
    process_name: String,
    up_bps: u64,
    down_bps: u64,
    connections: u32,
}

fn main() -> io::Result<()> {
    //funny_print();
    //listen_for_packets();

    // Open a connection to the SQLite database, creates if it doesnt exit
    let conn = match Connection::open("data.db") {
        Ok(conn) => conn,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to open SQLite database: {}", err),
            ));
        }
    };

    // Create the tables if they don't exist
    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS App (
            process_name TEXT PRIMARY KEY
        )",
        [],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create App table: {}", err),
        ));
    }
    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS processes (
            pid INTEGER primary key,
            process_name TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            connections INTEGER,
            constraint fk_processes_name foreign key (process_name) references App (process_name)
        )",
        [],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create processes table: {}", err),
        ));
    }

    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS connections (
            cid INTEGER PRIMARY KEY,
            source TEXT,
            destination TEXT,
            protocol TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            process_name TEXT
        )",
        [],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create connections table: {}", err),
        ));
    }

    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS remote_addresses (
            rid INTEGER PRIMARY KEY,
            address TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            connections INTEGER
        )",
        [],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create remote_addresses table: {}", err),
        ));
    }

    // test insert and select
    /*
    if let Err(err) = conn.execute(
        "INSERT INTO App (process_name) VALUES (?1)",
        params!["firefox"],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to insert into App table: {}", err),
        ));
    }

    let mut apps = Vec::new();

    // Execute a SELECT query to retrieve all rows from the App table
    let mut stmt = match conn.prepare("SELECT * FROM App") {
        Ok(stmt) => stmt,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to prepare SELECT query: {}", err),
            ));
        }
    };
    let rows = match stmt.query_map([], |row| {
        Ok(AppRow {
            process_name: row.get(0)?,
        })
    }) {
        Ok(rows) => rows,
        Err(err) => {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to execute SELECT query: {}", err),
            ));
        }
    };

    // Iterate over the Result object returned by query_map
    for row in rows {
        // Unwrap the Result object to get the App instance
        match row {
            Ok(app) => {
                apps.push(app);
            }
            Err(err) => eprintln!("Error retrieving row: {:?}", err),
        }
    }

    for app in &apps {
        println!("App process name: {}", app.process_name);
    } */

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
    //println!("Refresh block:\n{}", refresh_buffer);
    println!("---------------------------------------------------Processing refresh block---------------------------------------------------");
    parse_raw_block(refresh_buffer);
    Ok(())
}

fn parse_raw_block(raw_block: &str) {
    let process_re =
        Regex::new(r#"process: <(\d+)> "([^"]+)" up/down Bps: (\d+)/(\d+) connections: (\d+)"#)
            .unwrap();
    let connection_re = Regex::new(r#"connection: <(\d+)> <([^>]+)>:([^ ]+) => ([^:]+):(\d+) \(([^)]+)\) up/down Bps: (\d+)/(\d+) process: "([^"]+)""#).unwrap();
    let remote_address_re = Regex::new(
        r#"remote_address: <(\d+)> ([^ ]+) up/down Bps: (\d+)/(\d+) connections: (\d+)"#,
    )
    .unwrap();

    let mut processes: Vec<Process> = Vec::new();
    let mut connections: Vec<NetwrokConnection> = Vec::new();
    let mut remote_addresses: Vec<RemoteAddress> = Vec::new();

    for line in raw_block.lines() {
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
            let connection = NetwrokConnection {
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
        println!(
            "ID: {}, Name: {}, Up/Down Bps: {}/{}, Connections: {}",
            process.id, process.name, process.up_bps, process.down_bps, process.connections
        );
    }

    println!("\nConnections:");
    for connection in connections {
        println!(
            "ID: {}, Source: {}, Destination: {}, Protocol: {}, Up/Down Bps: {}/{}, Process: {}",
            connection.id,
            connection.source,
            connection.destination,
            connection.protocol,
            connection.up_bps,
            connection.down_bps,
            connection.process
        );
    }

    println!("\nRemote Addresses:");
    for remote_address in remote_addresses {
        println!(
            "ID: {}, Address: {}, Up/Down Bps: {}/{}, Connections: {}",
            remote_address.id,
            remote_address.address,
            remote_address.up_bps,
            remote_address.down_bps,
            remote_address.connections
        );
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
