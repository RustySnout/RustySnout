// sudo apt update
// sudo apt install libsqlite3-dev
// cargo install bandwhich
// sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep /path/to/bandwhich

// use sysinfo::Networks;

// use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self /*NetworkInterface*/};
// use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
// use pnet::packet::ipv4::Ipv4Packet;
// use pnet::packet::ipv6::Ipv6Packet;
// use pnet::packet::Packet;

//use pnet::packet::ip;
use regex::Regex;

use std::thread;

//use core::time;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};

use rusqlite::{params, Connection /* , Result*/};

use chrono::{/*DateTime,*/ Utc};

/*struct Process {
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

struct Interface {
    name: String,
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
}*/

use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct Process {
    id: String,
    name: String,
    up_bps: u64,
    down_bps: u64,
    connections: u32,
}

#[derive(Debug, Serialize)]
struct NetwrokConnection {
    id: String,
    source: String,
    destination: String,
    protocol: String,
    up_bps: u64,
    down_bps: u64,
    process: String,
}

#[derive(Debug, Serialize)]
struct RemoteAddress {
    id: String,
    address: String,
    up_bps: u64,
    down_bps: u64,
    connections: u32,
}

#[get("/")]
async fn index() -> impl Responder {
    // Retrieve data from the database
    let conn = Connection::open("data.db").expect("Failed to open database");

    // Retrieve processes from the database
    let processes = match conn.prepare("SELECT * FROM processes") {
        Ok(mut stmt) => {
            match stmt.query_map([], |row| {
                Ok(Process {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    up_bps: row.get(2)?,
                    down_bps: row.get(3)?,
                    connections: row.get(4)?,
                })
            }) {
                Ok(processes) => {
                    processes
                        .map(|p| p.unwrap()) // Unwrap each Result<Process, _>
                        .collect::<Vec<Process>>() // Collect into Vec<Process>
                }
                Err(err) => {
                    eprintln!("Error querying processes: {}", err);
                    return Err(io::Error::new(
                        io::ErrorKind::Other,
                        "Failed to query processes",
                    ));
                }
            }
        }
        Err(err) => {
            eprintln!("Error preparing statement for processes: {}", err);
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "Failed to prepare statement for processes",
            ));
        }
    };

    // Retrieve connections from the database
    let connections = conn
        .prepare("SELECT * FROM connections")
        .unwrap()
        .query_map([], |row| {
            Ok(NetwrokConnection {
                id: row.get(0)?,
                source: row.get(1)?,
                destination: row.get(2)?,
                protocol: row.get(3)?,
                up_bps: row.get(4)?,
                down_bps: row.get(5)?,
                process: row.get(6)?,
            })
        })
        .unwrap()
        .map(|c| c.unwrap())
        .collect::<Vec<NetwrokConnection>>();

    // Retrieve remote addresses from the database
    let remote_addresses = conn
        .prepare("SELECT * FROM remote_addresses")
        .unwrap()
        .query_map([], |row| {
            Ok(RemoteAddress {
                id: row.get(0)?,
                address: row.get(1)?,
                up_bps: row.get(2)?,
                down_bps: row.get(3)?,
                connections: row.get(4)?,
            })
        })
        .unwrap()
        .map(|r| r.unwrap())
        .collect::<Vec<RemoteAddress>>();

    // Render HTML template with data
    let html = format!(
        r#"
        <html>
            <head><title>Network Info</title></head>
            <body>
                <h1>Processes</h1>
                <ul>{}</ul>
                <h1>Connections</h1>
                <ul>{}</ul>
                <h1>Remote Addresses</h1>
                <ul>{}</ul>
            </body>
        </html>
        "#,
        processes
            .iter()
            .map(|p| format!("<li>{:?}</li>", p))
            .collect::<String>(),
        connections
            .iter()
            .map(|c| format!("<li>{:?}</li>", c))
            .collect::<String>(),
        remote_addresses
            .iter()
            .map(|r| format!("<li>{:?}</li>", r))
            .collect::<String>()
    );

    Ok(html)
}

async fn run_frontend() -> Result<(), std::io::Error> {
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8080")?
        .run()
        .await?;
    Ok(())
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
            process_name TEXT,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            constraint pk_app primary key (process_name, time, block_number)
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
            pid TEXT,
            process_name TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            connections INTEGER,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            constraint pk_processes primary key (pid, time, block_number),
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
        "CREATE TABLE IF NOT EXISTS interfaces (
            interface_name TEXT PRIMARY KEY,
            description TEXT,
            mac TEXT,
            flags TEXT
        )",
        [],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create interfaces table: {}", err),
        ));
    }

    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS interfacesIPS (
            interface_name TEXT,
            ips TEXT,
            FOREIGN KEY (interface_name) REFERENCES interfaces (interface_name),
            PRIMARY KEY (interface_name, ips)
        )",
        [],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create interfacesIPS table: {}", err),
        ));
    }

    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS connections (
            cid INTEGER,
            source TEXT,
            destination TEXT,
            protocol TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            process_name TEXT,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            CONSTRAINT pk_connections PRIMARY KEY (cid, time, block_number),
            CONSTRAINT fk_connections_source FOREIGN KEY (source) REFERENCES interfaces (interface_name)
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
            rid INTEGER,
            address TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            connections INTEGER,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            constraint pk_remote_addresses primary key (rid, time, block_number)
        )",
        [],
    ) {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to create remote_addresses table: {}", err),
        ));
    }

    // initialize the database with the interfaces
    let interfaces = datalink::interfaces();
    // Allow user to select interface
    for (i, interface) in interfaces.iter().enumerate() {
        println!("{}: {:?}", i, interface);
    }

    for interface in interfaces {
        if let Err(err) = conn.execute(
            "INSERT OR IGNORE INTO interfaces (interface_name, description, mac, flags) VALUES (?1, ?2, ?3, ?4)",
            params![interface.name, interface.description, interface.mac.unwrap().to_string(), interface.flags.to_string()],
        ) {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!("Failed to insert into interfaces table: {}", err),
            ));
        }
        // insert into interfacesIPS
        for ip in interface.ips {
            if let Err(err) = conn.execute(
                "INSERT OR IGNORE INTO interfacesIPS (interface_name, ips) VALUES (?1, ?2)",
                params![interface.name, ip.to_string()],
            ) {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to insert into interfacesIPS table: {}", err),
                ));
            }
        }
    }

    let future_result = run_frontend(); // Obtain the Future
    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(future_result); // Await the Future

    if let Err(err) = result {
        eprintln!("Error running frontend: {}", err);
    }

    let mut child = Command::new("bandwhich")
        .arg("--raw")
        .stdout(Stdio::piped())
        .spawn()?;

    let output = child.stdout.take().expect("Failed to open stdout");
    let reader = BufReader::new(output);

    let mut refresh_buffer = String::new();
    let mut block_number = 0;
    // Process each line of the output in a loop
    for line in reader.lines() {
        let line = line?;
        // Detect the start of a new refresh block
        if line.starts_with("Refreshing:") {
            // Process the previous refresh block, if any
            if !refresh_buffer.is_empty() {
                process_refresh_buffer(&refresh_buffer, block_number)?;
                block_number += 1;
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
        process_refresh_buffer(&refresh_buffer, block_number)?;
    }

    Ok(())
}

fn process_refresh_buffer(refresh_buffer: &str, block_number: i32) -> io::Result<()> {
    // Process the refresh block here
    //println!("Refresh block:\n{}", refresh_buffer);
    println!("---------------------------------------------------Processing refresh block---------------------------------------------------");
    parse_and_save_raw_block(refresh_buffer, block_number);
    Ok(())
}

fn parse_and_save_raw_block(raw_block: &str, block_number: i32) {
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

    // println!("\nProcesses:");
    // for process in processes {
    //     println!(
    //         "ID: {}, Name: {}, Up/Down Bps: {}/{}, Connections: {}",
    //         process.id, process.name, process.up_bps, process.down_bps, process.connections
    //     );
    // }

    // println!("\nConnections:");
    // for connection in connections {
    //     println!(
    //         "ID: {}, Source: {}, Destination: {}, Protocol: {}, Up/Down Bps: {}/{}, Process: {}",
    //         connection.id,
    //         connection.source,
    //         connection.destination,
    //         connection.protocol,
    //         connection.up_bps,
    //         connection.down_bps,
    //         connection.process
    //     );
    // }

    // println!("\nRemote Addresses:");
    // for remote_address in remote_addresses {
    //     println!(
    //         "ID: {}, Address: {}, Up/Down Bps: {}/{}, Connections: {}",
    //         remote_address.id,
    //         remote_address.address,
    //         remote_address.up_bps,
    //         remote_address.down_bps,
    //         remote_address.connections
    //     );
    // }

    // add to tables
    let current_time = Utc::now().timestamp_millis();

    let conn = match Connection::open("data.db") {
        Ok(conn) => conn,
        Err(err) => {
            eprintln!("Failed to open SQLite database: {}", err);
            return;
        }
    };

    for process in processes {
        // insert into app
        if let Err(err) = conn.execute(
            "INSERT OR IGNORE INTO App (process_name, time, block_number) VALUES (?1, ?2, ?3)",
            params![process.name, current_time, block_number],
        ) {
            eprintln!("Failed to insert into App table: {}", err);
        }

        // insert into processes
        if let Err(err) = conn.execute(
            "INSERT OR IGNORE INTO processes (pid, process_name, up_bps, down_bps, connections, time, block_number) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![process.id, process.name, process.up_bps, process.down_bps, process.connections, current_time, block_number],
        ) {
            eprintln!("Failed to insert {} {current_time} {block_number} into processes table: {}",process.id, err);
        }
    }

    for connection in connections {
        // insert into Connections
        if let Err(err) = conn.execute(
            "INSERT OR IGNORE INTO connections (cid, source, destination, protocol, up_bps, down_bps, process_name, time, block_number) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![connection.id,
                    connection.source,
                    connection.destination,
                    connection.protocol,
                    connection.up_bps,
                    connection.down_bps,
                    connection.process,
                    current_time,
                    block_number],
        ) {
            eprintln!("Failed to insert into connections table: {}", err);
        }
    }

    for remote_address in remote_addresses {
        // insert into remote_addresses
        if let Err(err) = conn.execute(
            "INSERT OR IGNORE INTO remote_addresses (rid, address, up_bps, down_bps, connections, time, block_number) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![remote_address.id,
                    remote_address.address,
                    remote_address.up_bps,
                    remote_address.down_bps,
                    remote_address.connections,
                    current_time,
                    block_number],
        ) {
            eprintln!("Failed to insert into remote_addresses table: {}", err);
        }
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
