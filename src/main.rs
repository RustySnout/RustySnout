// sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep /path/to/bandwhich
mod dns;

use anyhow::{anyhow, bail};
use ipnetwork::IpNetwork;
use itertools::Itertools;
use pnet::datalink::{self, Channel::Ethernet, Config, DataLinkReceiver, NetworkInterface};
use rusqlite::{params, Connection as sqlConnection /* , Result*/};
use std::{
    collections::HashMap,
    fmt,
    io::{self, ErrorKind, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
    time::{Duration, Instant},
};
use tokio::runtime::Runtime;

use procfs::process::FDTarget;

//use thiserror::Error;

pub struct OsInputOutput {
    pub interfaces_with_frames: Vec<(NetworkInterface, Box<dyn DataLinkReceiver>)>,
    pub get_open_sockets: fn() -> OpenSockets,
    pub dns_client: Option<dns::Client>,
    pub write_to_stdout: Box<dyn FnMut(String) + Send>,
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Debug, Copy)]
pub enum Protocol {
    Tcp,
    Udp,
}

impl Protocol {
    #[allow(dead_code)]
    pub fn from_str(string: &str) -> Option<Self> {
        match string {
            "TCP" => Some(Protocol::Tcp),
            "UDP" => Some(Protocol::Udp),
            _ => None,
        }
    }
}

impl fmt::Display for Protocol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Protocol::Tcp => write!(f, "tcp"),
            Protocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Hash, Copy)]
pub struct Socket {
    pub ip: IpAddr,
    pub port: u16,
}

impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Socket { ip, port } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "[{v6}]:{port}"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct LocalSocket {
    pub ip: IpAddr,
    pub port: u16,
    pub protocol: Protocol,
}

impl fmt::Debug for LocalSocket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let LocalSocket { ip, port, protocol } = self;
        match ip {
            IpAddr::V4(v4) => write!(f, "{protocol}://{v4}:{port}"),
            IpAddr::V6(v6) => write!(f, "{protocol}://[{v6}]:{port}"),
        }
    }
}

#[derive(PartialEq, Hash, Eq, Clone, PartialOrd, Ord, Copy)]
pub struct Connection {
    pub remote_socket: Socket,
    pub local_socket: LocalSocket,
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Connection {
            remote_socket,
            local_socket,
        } = self;
        write!(f, "{local_socket:?} => {remote_socket:?}")
    }
}

pub fn display_ip_or_host(ip: IpAddr, ip_to_host: &HashMap<IpAddr, String>) -> String {
    match ip_to_host.get(&ip) {
        Some(host) => host.clone(),
        None => ip.to_string(),
    }
}

pub fn display_connection_string(
    connection: &Connection,
    ip_to_host: &HashMap<IpAddr, String>,
    interface_name: &str,
) -> String {
    format!(
        "<{interface_name}>:{} => {}:{} ({})",
        connection.local_socket.port,
        display_ip_or_host(connection.remote_socket.ip, ip_to_host),
        connection.remote_socket.port,
        connection.local_socket.protocol,
    )
}

impl Connection {
    pub fn new(
        remote_socket: SocketAddr,
        local_ip: IpAddr,
        local_port: u16,
        protocol: Protocol,
    ) -> Self {
        Connection {
            remote_socket: Socket {
                ip: remote_socket.ip(),
                port: remote_socket.port(),
            },
            local_socket: LocalSocket {
                ip: local_ip,
                port: local_port,
                protocol,
            },
        }
    }
}

#[derive(Clone)]
pub struct ConnectionInfo {
    pub interface_name: String,
    pub total_bytes_downloaded: u128,
    pub total_bytes_uploaded: u128,
}

#[derive(PartialEq, Hash, Eq, Debug, Clone, PartialOrd)]
pub enum Direction {
    Download,
    Upload,
}

impl Direction {
    pub fn new(network_interface_ips: &[IpNetwork], source: IpAddr) -> Self {
        if network_interface_ips
            .iter()
            .any(|ip_network| ip_network.ip() == source)
        {
            Direction::Upload
        } else {
            Direction::Download
        }
    }
}

#[derive(Debug)]
pub struct Segment {
    pub interface_name: String,
    pub connection: Connection,
    pub direction: Direction,
    pub data_length: u128,
}

#[derive(Clone)]
pub struct Utilization {
    pub connections: HashMap<Connection, ConnectionInfo>,
}

impl Utilization {
    pub fn new() -> Self {
        let connections = HashMap::new();
        Utilization { connections }
    }
    pub fn clone_and_reset(&mut self) -> Self {
        let clone = self.clone();
        self.connections.clear();
        clone
    }
    pub fn update(&mut self, seg: Segment) {
        let total_bandwidth = self
            .connections
            .entry(seg.connection)
            .or_insert(ConnectionInfo {
                interface_name: seg.interface_name,
                total_bytes_downloaded: 0,
                total_bytes_uploaded: 0,
            });
        match seg.direction {
            Direction::Download => {
                total_bandwidth.total_bytes_downloaded += seg.data_length;
            }
            Direction::Upload => {
                total_bandwidth.total_bytes_uploaded += seg.data_length;
            }
        }
    }
}

pub struct OpenSockets {
    sockets_to_procs: HashMap<LocalSocket, ProcessInfo>,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
}
impl ProcessInfo {
    pub fn new(name: &str, pid: u32) -> Self {
        Self {
            name: name.to_string(),
            pid,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, thiserror::Error)]
pub enum GetInterfaceError {
    #[error("Permission error: {0}")]
    PermissionError(String),
    #[error("Other error: {0}")]
    OtherError(String),
}

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

fn main() -> anyhow::Result<()> {
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
            constraint pk_app primary key (process_name, time, block_number)
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
            constraint pk_processes primary key (pid, time, block_number),
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
            CONSTRAINT pk_connections PRIMARY KEY (cid, time, block_number),
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
            constraint pk_remote_addresses primary key (rid, time, block_number)
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
    //let ui_offset = Arc::new(AtomicUsize::new(0));
    let dns_shown = true;

    let mut active_threads = vec![];

    let get_open_sockets = os_input.get_open_sockets;
    let mut write_to_stdout = os_input.write_to_stdout;
    let mut dns_client = os_input.dns_client;

    let network_utilization = Arc::new(Mutex::new(Utilization::new()));

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

fn get_interface(interface_name: &str) -> Option<NetworkInterface> {
    datalink::interfaces()
        .into_iter()
        .find(|iface| iface.name == interface_name)
}

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
