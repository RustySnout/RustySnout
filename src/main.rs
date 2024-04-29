// sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep /path/to/bandwhich

use pnet::datalink::{self /*NetworkInterface*/};
use rusqlite::{params, Connection /* , Result*/};
use std::fmt;
use std::io;

use std::collections::HashMap;

use procfs::process::FDTarget;

use std::net::IpAddr;

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

fn main() -> io::Result<()> {
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

    // Get the open sockets
    let open_sockets = get_open_sockets();

    // Print the open sockets
    for (socket, proc_info) in open_sockets.sockets_to_procs {
        let ip = match socket.ip {
            IpAddr::V4(ip) => ip.to_string(),
            IpAddr::V6(ip) => ip.to_string(),
        };
        let port = socket.port;
        let protocol = match socket.protocol {
            Protocol::Tcp => "TCP",
            Protocol::Udp => "UDP",
        };
        let process_name = proc_info.name;
        let pid = proc_info.pid;

        println!(
            "Socket: {}:{} Protocol: {} Process: {} PID: {}",
            ip, port, protocol, process_name, pid
        );
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
