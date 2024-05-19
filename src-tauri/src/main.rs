// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

// sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep target/debug/rustysnout
mod dns;
mod mystate;
mod objects;
mod sniffer;
mod interface_throttling;
mod process_throttling;
pub use mystate::*;
pub use objects::{
    GetInterfaceError, IpTable, LocalSocket, MyState, OpenSockets, OsInputOutput, ProcessInfo,
    Protocol, Utilization,
};
use serde_json::json;
use sniffer::Sniffer;

use anyhow::{anyhow, bail};
use itertools::Itertools;
use pnet::datalink::{self, Channel::Ethernet, Config, DataLinkReceiver, NetworkInterface};
use procfs::process::FDTarget;
use rusqlite::{params, Connection as sqlConnection /* , Result*/};
use std::{
    collections::HashMap,
    io::{self, ErrorKind, Write},
    net::Ipv4Addr,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    thread::{self, park_timeout},
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;
use rusqlite::{Connection, Result};
fn main() {

  // if data.db exists nuke it
  nuke_database();

  // create database before threads are initiated
  create_database_ifn_exist().expect("Failed to create database");

  // start_monitoring in a separate thread
  thread::spawn(|| {
    let _ = start_monitoring();
  });

  // Run the Tauri app in the main thread
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
        get_process_wrapper,
        get_remote_address_wrapper,
        get_connections_wrapper,
        get_interfaces_wrapper,
        get_current_throughput_wrapper,
        throttle_ip_wrapper,
        remove_interface_throttling_wrapper,
        interface_throttling_wrapper,
        launch_throttled_app_wrapper
        ])
    .build(tauri::generate_context!())
    .expect("error while building tauri application")
    .run(|_app_handle, event| match event { //this is done to prevent backend from exiting so it keeps monitoring
      tauri::RunEvent::ExitRequested { api, .. } => {
        api.prevent_exit();
      }
      _ => {}
    });

}


/* querying data from sqlite database data.db
   after we have the query functions we will wrap them in the tauri
   command macro so we can invoke them from the javascript 
   we return a the query result as a json string
*/

// gets the latest process block row and then return json string
#[tauri::command]
async fn get_remote_address_wrapper() -> String {
  match get_remote_address() {
    Ok(result) => result,
    Err(err) => format!("Error: {}", err),
  }
}

#[tauri::command]
async fn get_process_wrapper() -> String {
  match get_process() {
    Ok(result) => result,
    Err(err) => format!("Error: {}", err),
  }
}

#[tauri::command]
async fn get_connections_wrapper() -> String {
  match get_connections() {
    Ok(result) => result,
    Err(err) => format!("Error: {}", err),
  }
}

#[tauri::command]
async fn get_current_throughput_wrapper() -> String {
  match get_current_throughput() {
    Ok(result) => result,
    Err(err) => format!("Error: {}", err),
  }
}

#[tauri::command]
fn throttle_ip_wrapper(ip: &str, rate: &str, interface: &str) {
  process_throttling::throttle_ip(ip, rate, interface);
}

#[tauri::command]
fn remove_interface_throttling_wrapper(interface: &str) {
  interface_throttling::remove_interface_throttling(interface);
}

#[tauri::command]
fn interface_throttling_wrapper(interface: &str, bandwidth_limit: u32) {
  interface_throttling::interface_throttling(interface, bandwidth_limit);
}

fn get_current_throughput() -> Result<String, anyhow::Error> {
  let conn = Connection::open("data.db")?;
  let mut stmt = conn.prepare("SELECT up_bps, down_bps FROM processes WHERE block_number = (SELECT MAX(block_number) FROM processes)")?;

  let throughput_iter = stmt.query_map([], |row| {
    Ok(json!({
      "up_bps": row.get::<_, i64>(0)?,
      "down_bps": row.get::<_, i64>(1)?,
    }))
  })?;

  let mut throughput_vec = Vec::new();

  for throughput in throughput_iter {
    throughput_vec.push(throughput?);
  }

  Ok(json!(throughput_vec).to_string())
}

fn get_remote_address() -> Result<String, anyhow::Error> {

  let conn = Connection::open("data.db")?;
  let mut stmt = conn.prepare("SELECT rid, address, up_bps, down_bps, connections FROM remote_addresses WHERE block_number = (SELECT MAX(block_number) FROM remote_addresses)")?;

  let connections_stats_iter = stmt.query_map([], |row| {
    Ok(json!({
      "rid": row.get::<_, String>(0)?,
      "address": row.get::<_, String>(1)?,
      "up_bps": row.get::<_, i64>(2)?,
      "down_bps": row.get::<_, i64>(3)?,
      "connections": row.get::<_, i64>(4)?,
    }))
  })?;

  let mut connections_stats_vec = Vec::new();

  for connections_stats in connections_stats_iter {
    connections_stats_vec.push(connections_stats?);
  }

  Ok(json!(connections_stats_vec).to_string())

}

fn get_connections() -> Result<String, anyhow::Error> {

  let conn = Connection::open("data.db")?;
  let mut stmt = conn.prepare("SELECT source, destination, protocol, up_bps, down_bps, process_name FROM connections WHERE block_number = (SELECT MAX(block_number) FROM connections)")?;

  let connections_iter = stmt.query_map([], |row| {
    Ok(json!({
      "source": row.get::<_, String>(0)?,
      "destination": row.get::<_, String>(1)?,
      "protocol": row.get::<_, String>(2)?,
      "up_bps": row.get::<_, i64>(3)?,
      "down_bps": row.get::<_, i64>(4)?,
      "process_name": row.get::<_, String>(5)?,
    }))
  })?;

  let mut connections_vec = Vec::new();

  for connection in connections_iter {
    connections_vec.push(connection?);
  }

  Ok(json!(connections_vec).to_string())

}

#[tauri::command]
fn get_interfaces_wrapper() -> String {
  match get_interfaces() {
    Ok(result) => result,
    Err(err) => format!("Error: {}", err),
  }
}

#[tauri::command]
fn launch_throttled_app_wrapper(interface: String, up_limit: u32, down_limit: u32, app: String) {
  interface_throttling::launch_throttle_app(up_limit, down_limit, app);
}

fn get_interfaces() -> Result<String, anyhow::Error> {

  let conn = Connection::open("data.db")?;
  let mut stmt = conn.prepare("SELECT interface_name, mac FROM interfaces")?;

  let interfaces_iter = stmt.query_map([], |row| {
    Ok(json!({
      "interface_name": row.get::<_, String>(0)?,
      "mac": row.get::<_, String>(1)?,
    }))
  })?;

  let mut interfaces_vec = Vec::new();

  for interface in interfaces_iter {
    interfaces_vec.push(interface?);
  }

  Ok(json!(interfaces_vec).to_string())
}

fn get_process() -> Result<String, anyhow::Error> {

  let conn = Connection::open("data.db")?;
  let mut stmt = conn.prepare("SELECT pid, process_name, up_bps, down_bps, connections FROM processes WHERE block_number = (SELECT MAX(block_number) FROM processes)")?;

  let process_stats_iter = stmt.query_map([], |row| {
    Ok(json!({
      "pid": row.get::<_, i64>(0)?,
      "process_name": row.get::<_, String>(1)?,
      "up_bps": row.get::<_, i64>(2)?,
      "down_bps": row.get::<_, i64>(3)?,
      "connections": row.get::<_, i64>(4)?,
    }))
  })?;

  let mut process_stats_vec = Vec::new();

  for process_stats in process_stats_iter {
    process_stats_vec.push(process_stats?);
  }

  Ok(json!(process_stats_vec).to_string())

}



//use thiserror::Error;
const DISPLAY_DELTA: Duration = Duration::from_millis(1000);

fn create_database_ifn_exist() -> anyhow::Result<()> {

    let conn = match sqlConnection::open("data.db") {
        Ok(conn) => conn,
        Err(err) => {
            return Err(anyhow::Error::msg(format!(
                "Failed to open SQLite database: {}",
                err
            )));
        }
    };


    // Create the tables if they don't exist
    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS App (
            process_name TEXT,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            constraint pk_app primary key (process_name, block_number)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!(
            "Failed to create App table: {}",
            err
        )));
    }
    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS processes (
            pid INTEGER,
            process_name TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            connections INTEGER,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            constraint pk_processes primary key (pid, block_number),
            constraint fk_processes_name foreign key (process_name) references App (process_name)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!(
            "Failed to create processes table: {}",
            err
        )));
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
        return Err(anyhow::Error::msg(format!(
            "Failed to create interfaces table: {}",
            err
        )));
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
        return Err(anyhow::Error::msg(format!(
            "Failed to create interfacesIPS table: {}",
            err
        )));
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
            CONSTRAINT pk_connections PRIMARY KEY (cid, block_number),
            CONSTRAINT fk_connections_source FOREIGN KEY (source) REFERENCES interfaces (interface_name)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!("Failed to create connections table: {}", err),
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
            constraint pk_remote_addresses primary key (rid, block_number)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!(
            "Failed to create remote_addresses table: {}",
            err
        )));
    }

    Ok(())

}

fn nuke_database() {
  //delete data.db
  std::fs::remove_file("data.db").expect("Failed to delete database");
}

fn start_monitoring() -> anyhow::Result<()> {
    // Open a connection to the SQLite database, creates if it doesnt exit
    let conn = match sqlConnection::open("data.db") {
        Ok(conn) => conn,
        Err(err) => {
            return Err(anyhow::Error::msg(format!(
                "Failed to open SQLite database: {}",
                err
            )));
        }
    };

    // Create the tables if they don't exist
    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS App (
            process_name TEXT,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            constraint pk_app primary key (process_name, block_number)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!(
            "Failed to create App table: {}",
            err
        )));
    }
    if let Err(err) = conn.execute(
        "CREATE TABLE IF NOT EXISTS processes (
            pid INTEGER,
            process_name TEXT,
            up_bps INTEGER,
            down_bps INTEGER,
            connections INTEGER,
            time INTEGER DEFAULT CURRENT_TIMESTAMP,
            block_number INTEGER,
            constraint pk_processes primary key (pid, block_number),
            constraint fk_processes_name foreign key (process_name) references App (process_name)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!(
            "Failed to create processes table: {}",
            err
        )));
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
        return Err(anyhow::Error::msg(format!(
            "Failed to create interfaces table: {}",
            err
        )));
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
        return Err(anyhow::Error::msg(format!(
            "Failed to create interfacesIPS table: {}",
            err
        )));
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
            CONSTRAINT pk_connections PRIMARY KEY (cid, block_number),
            CONSTRAINT fk_connections_source FOREIGN KEY (source) REFERENCES interfaces (interface_name)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!("Failed to create connections table: {}", err),
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
            constraint pk_remote_addresses primary key (rid, block_number)
        )",
        [],
    ) {
        return Err(anyhow::Error::msg(format!(
            "Failed to create remote_addresses table: {}",
            err
        )));
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
            return Err(anyhow::Error::msg(format!("Failed to insert into interfaces table: {}", err),
            ));
        }
        // insert into interfacesIPS
        for ip in interface.ips {
            if let Err(err) = conn.execute(
                "INSERT OR IGNORE INTO interfacesIPS (interface_name, ips) VALUES (?1, ?2)",
                params![interface.name, ip.to_string()],
            ) {
                return Err(anyhow::Error::msg(format!(
                    "Failed to insert into interfacesIPS table: {}",
                    err
                )));
            }
        }
    }

    let os_input = get_input(datalink::interfaces(), None)?;

    let running = Arc::new(AtomicBool::new(true));
    let paused = Arc::new(AtomicBool::new(false));
    let last_start_time = Arc::new(RwLock::new(Instant::now()));
    let cumulative_time = Arc::new(RwLock::new(Duration::new(0, 0)));
    let state_offset = Arc::new(AtomicUsize::new(0));

    let mut active_threads = vec![];

    let get_open_sockets = os_input.get_open_sockets;
    let mut write_to_stdout = os_input.write_to_stdout;
    let mut dns_client = os_input.dns_client;

    let network_utilization = Arc::new(Mutex::new(Utilization::new()));
    let mystate = Arc::new(Mutex::new(MyState::new()));

    // NEED UI

    let display_handler = thread::Builder::new()
        .name("display_handler".to_string())
        .spawn({
            let running = running.clone();
            let paused = paused.clone();
            let state_offset = state_offset.clone();

            let network_utilization = network_utilization.clone();
            let last_start_time = last_start_time.clone();
            let cumulative_time = cumulative_time.clone();
            let mystate = mystate.clone();

            move || {
                while running.load(Ordering::Acquire) {
                    let render_start_time = Instant::now();
                    let utilization = { network_utilization.lock().unwrap().clone_and_reset() };
                    let OpenSockets { sockets_to_procs } = get_open_sockets();

                    // Attempt to resolve IPs to hostnames
                    let mut ip_to_host = IpTable::new();
                    if let Some(dns_client) = dns_client.as_mut() {
                        ip_to_host = dns_client.cache();
                        let unresolved_ips = utilization
                            .connections
                            .keys()
                            .filter(|conn| !ip_to_host.contains_key(&conn.remote_socket.ip))
                            .map(|conn| conn.remote_socket.ip)
                            .collect::<Vec<_>>();
                        dns_client.resolve(unresolved_ips);
                    }
                    {
                        let mut mystate = mystate.lock().unwrap();
                        let paused = paused.load(Ordering::SeqCst);
                        let state_offset = state_offset.load(Ordering::SeqCst);
                        if !paused {
                            mystate.update_state(sockets_to_procs, utilization, ip_to_host);
                        }
                        let elapsed_time = elapsed_time(
                            *last_start_time.read().unwrap(),
                            *cumulative_time.read().unwrap(),
                            paused,
                        );

                        // TODO: SAVE TO SQL DATABASE AND PRINT TO STDOUT PLS
                        mystate.output_text(&mut write_to_stdout);
                    }
                    let render_duration = render_start_time.elapsed();
                    if render_duration < DISPLAY_DELTA {
                        park_timeout(DISPLAY_DELTA - render_duration);
                    }
                }
            }
        })
        .unwrap();

    active_threads.push(display_handler);
    // TODO: do we need terminal event handler?

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

pub fn get_open_sockets() -> OpenSockets {
    let mut open_sockets = HashMap::new();
    let mut inode_to_proc = HashMap::new();

    if let Ok(all_procs) = procfs::process::all_processes() {
        for process in all_procs.filter_map(|res| res.ok()) {
            let Ok(fds) = process.fd() else { continue };
            let Ok(stat) = process.stat() else { continue };
            let proc_name = stat.comm;
            let proc_info = ProcessInfo::new(&proc_name, stat.pid as u32);
            for fd in fds.filter_map(|res| res.ok()) {
                if let FDTarget::Socket(inode) = fd.target {
                    inode_to_proc.insert(inode, proc_info.clone());
                }
            }
        }
    }

    macro_rules! insert_proto {
        ($source: expr, $proto: expr) => {
            let entries = $source.into_iter().filter_map(|res| res.ok()).flatten();
            for entry in entries {
                if let Some(proc_info) = inode_to_proc.get(&entry.inode) {
                    let socket = LocalSocket {
                        ip: entry.local_address.ip(),
                        port: entry.local_address.port(),
                        protocol: $proto,
                    };
                    open_sockets.insert(socket, proc_info.clone());
                }
            }
        };
    }

    insert_proto!([procfs::net::tcp(), procfs::net::tcp6()], Protocol::Tcp);
    insert_proto!([procfs::net::udp(), procfs::net::udp6()], Protocol::Udp);

    OpenSockets {
        sockets_to_procs: open_sockets,
    }
}

pub fn get_datalink_channel(
    interface: &NetworkInterface,
) -> Result<Box<dyn DataLinkReceiver>, GetInterfaceError> {
    let config = Config {
        read_timeout: Some(Duration::new(1, 0)),
        read_buffer_size: 65536,
        ..Default::default()
    };

    match datalink::channel(interface, config) {
        Ok(Ethernet(_tx, rx)) => Ok(rx),
        Ok(_) => Err(GetInterfaceError::OtherError(format!(
            "{}: Unsupported interface type",
            interface.name
        ))),
        Err(e) => match e.kind() {
            ErrorKind::PermissionDenied => Err(GetInterfaceError::PermissionError(
                interface.name.to_owned(),
            )),
            _ => Err(GetInterfaceError::OtherError(format!(
                "{}: {e}",
                &interface.name
            ))),
        },
    }
}

// fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
//     datalink::interfaces()
//         .into_iter()
//         .find(|iface| iface.name == interface_name)
// }

fn create_write_to_stdout() -> Box<dyn FnMut(String) + Send> {
    let mut stdout = io::stdout();
    Box::new({
        move |output: String| {
            writeln!(stdout, "{}", output).unwrap();
        }
    })
}

fn eperm_message() -> &'static str {
    r#"
    Insufficient permissions to listen on network interface(s). You can work around
    this issue like this:

    * Try running `rustysnout` with `sudo`

    * Build a `setcap(8)` wrapper for `rustysnout` with the following rules:
        `cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep`
    "#
}

pub fn get_input(
    interfaces: Vec<NetworkInterface>,
    dns_server: Option<Ipv4Addr>,
) -> anyhow::Result<OsInputOutput> {
    // get the user's requested interface, if any
    // IDEA: allow requesting multiple interfaces

    // take the user's requested interfaces (or all interfaces), and filter for up ones
    let available_interfaces = interfaces
        .into_iter()
        .filter(|interface| {
            // see https://github.com/libpnet/libpnet/issues/564
            let keep = interface.is_up() && !interface.ips.is_empty();
            if !keep {
                println!("{} is down. Skipping it.", interface.name);
            }
            keep
        })
        .collect_vec();

    // bail if no interfaces are up
    if available_interfaces.is_empty() {
        bail!("Failed to find any network interface to listen on.");
    }

    // try to get a frame receiver for each interface
    let interfaces_with_frames_res = available_interfaces
        .into_iter()
        .map(|interface| {
            let frames_res = get_datalink_channel(&interface);
            (interface, frames_res)
        })
        .collect_vec();

    // warn for all frame receivers we failed to acquire
    interfaces_with_frames_res
        .iter()
        .filter_map(|(interface, frames_res)| frames_res.as_ref().err().map(|err| (interface, err)))
        .for_each(|(interface, err)| {
            println!(
                "Failed to acquire a frame receiver for {}: {err}",
                interface.name
            )
        });

    if interfaces_with_frames_res
        .iter()
        .all(|(_, frames)| frames.is_err())
    {
        let (permission_err_interfaces, other_errs) = interfaces_with_frames_res.iter().fold(
            (vec![], vec![]),
            |(mut perms, mut others), (_, res)| {
                match res {
                    Ok(_) => (),
                    Err(GetInterfaceError::PermissionError(interface)) => {
                        perms.push(interface.as_str())
                    }
                    Err(GetInterfaceError::OtherError(err)) => others.push(err.as_str()),
                }
                (perms, others)
            },
        );

        let err_msg = match (permission_err_interfaces.is_empty(), other_errs.is_empty()) {
            (false, false) => format!(
                "\n\n{}: {}\nAdditional errors:\n{}",
                permission_err_interfaces.join(", "),
                eperm_message(),
                other_errs.join("\n")
            ),
            (false, true) => format!(
                "\n\n{}: {}",
                permission_err_interfaces.join(", "),
                eperm_message()
            ),
            (true, false) => format!("\n\n{}", other_errs.join("\n")),
            (true, true) => unreachable!("Found no errors in error handling code path."),
        };
        bail!(err_msg);
    }

    // filter out interfaces for which we failed to acquire a frame receiver
    let interfaces_with_frames = interfaces_with_frames_res
        .into_iter()
        .filter_map(|(interface, res)| res.ok().map(|frames| (interface, frames)))
        .collect();

    let dns_client = {
        let runtime = Runtime::new()?;
        let resolver = runtime
            .block_on(dns::Resolver::new(dns_server))
            .map_err(|err| {
                anyhow!("Could not initialize the DNS resolver. Are you offline?\n\nReason: {err}")
            })?;
        let dns_client = dns::Client::new(resolver, runtime)?;
        Some(dns_client)
    };

    let write_to_stdout = create_write_to_stdout();

    Ok(OsInputOutput {
        interfaces_with_frames,
        get_open_sockets,
        dns_client,
        write_to_stdout,
    })
}

pub fn elapsed_time(last_start_time: Instant, cumulative_time: Duration, paused: bool) -> Duration {
    if paused {
        cumulative_time
    } else {
        cumulative_time + last_start_time.elapsed()
    }
}
