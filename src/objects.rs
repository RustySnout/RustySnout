use crate::dns;
use crate::mystate::MYState;
use chrono::prelude::*;
use clap::ValueEnum;
use derivative::Derivative;
use ipnetwork::IpNetwork;
use pnet::datalink::{self, Channel::Ethernet, Config, DataLinkReceiver, NetworkInterface};
use std::{
    collections::HashMap,
    fmt,
    io::ErrorKind,
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use strum::EnumIter;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, ValueEnum, EnumIter)]
pub enum UnitFamily {
    #[default]
    /// bytes, in powers of 2^10
    BinBytes,
    /// bits, in powers of 2^10
    BinBits,
    /// bytes, in powers of 10^3
    SiBytes,
    /// bits, in powers of 10^3
    SiBits,
}

pub struct MyState {
    state: MYState, // TODO: Do we need this? wait for weso
    ip_to_host: HashMap<IpAddr, String>,
}

impl MyState {
    pub fn new() -> Self {
        let state = {
            let mut state = MYState::default();
            state.interface_name = None;
            state.unit_family = BandwidthUnitFamily::default();
            state.cumulative_mode = false; // TODO: Do we need this? wait for weso and allow for ui modification of this
            state
        };
        MyState {
            state,
            ip_to_host: HashMap::new(),
        }
    }
    pub fn output_text(&mut self, write_to_stdout: &mut (dyn FnMut(String) + Send)) {
        let state = &self.state;
        let ip_to_host = &self.ip_to_host;
        let local_time: DateTime<Local> = Local::now();
        let timestamp = local_time.timestamp();
        let mut no_traffic = true;

        let output_process_data = |write_to_stdout: &mut (dyn FnMut(String) + Send),
                                   no_traffic: &mut bool| {
            for (proc_info, process_network_data) in &state.processes {
                write_to_stdout(format!(
                    "process: <{timestamp}> \"{}\" up/down Bps: {}/{} connections: {}",
                    proc_info.name,
                    process_network_data.total_bytes_uploaded,
                    process_network_data.total_bytes_downloaded,
                    process_network_data.connection_count
                ));
                *no_traffic = false;
            }
        };

        let output_connections_data =
            |write_to_stdout: &mut (dyn FnMut(String) + Send), no_traffic: &mut bool| {
                for (connection, connection_network_data) in &state.connections {
                    write_to_stdout(format!(
                        "connection: <{timestamp}> {} up/down Bps: {}/{} process: \"{}\"",
                        display_connection_string(
                            connection,
                            ip_to_host,
                            &connection_network_data.interface_name,
                        ),
                        connection_network_data.total_bytes_uploaded,
                        connection_network_data.total_bytes_downloaded,
                        connection_network_data.process_name
                    ));
                    *no_traffic = false;
                }
            };

        let output_adressess_data = |write_to_stdout: &mut (dyn FnMut(String) + Send),
                                     no_traffic: &mut bool| {
            for (remote_address, remote_address_network_data) in &state.remote_addresses {
                write_to_stdout(format!(
                    "remote_address: <{timestamp}> {} up/down Bps: {}/{} connections: {}",
                    display_ip_or_host(*remote_address, ip_to_host),
                    remote_address_network_data.total_bytes_uploaded,
                    remote_address_network_data.total_bytes_downloaded,
                    remote_address_network_data.connection_count
                ));
                *no_traffic = false;
            }
        };

        // header
        write_to_stdout("Refreshing:".into());

        // body1
        output_process_data(write_to_stdout, &mut no_traffic);
        output_connections_data(write_to_stdout, &mut no_traffic);
        output_adressess_data(write_to_stdout, &mut no_traffic);

        // body2: In case no traffic is detected
        if no_traffic {
            write_to_stdout("<NO TRAFFIC>".into());
        }

        // footer
        write_to_stdout("".into());
    }
    pub fn update_state(
        &mut self,
        connections_to_procs: HashMap<LocalSocket, ProcessInfo>,
        utilization: Utilization,
        ip_to_host: HashMap<IpAddr, String>,
    ) {
        self.state.update(connections_to_procs, utilization);
        self.ip_to_host.extend(ip_to_host);
    }
}

pub type IpTable = HashMap<IpAddr, String>;

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
    pub sockets_to_procs: HashMap<LocalSocket, ProcessInfo>,
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

#[derive(Copy, Clone, Derivative, Default, Eq, PartialEq)]
#[derivative(Debug = "transparent")]
pub struct BandwidthUnitFamily(UnitFamily);
impl From<UnitFamily> for BandwidthUnitFamily {
    fn from(value: UnitFamily) -> Self {
        Self(value)
    }
}
impl BandwidthUnitFamily {
    #[inline]
    /// Returns an array of tuples, corresponding to the steps of this unit family.
    ///
    /// Each step contains a divisor, an upper bound, and a unit suffix.
    fn steps(&self) -> [(f64, f64, &'static str); 6] {
        /// The fraction of the next unit the value has to meet to step up.
        const STEP_UP_FRAC: f64 = 0.95;
        /// Binary base: 2^10.
        const BB: f64 = 1024.0;

        use UnitFamily as F;
        // probably could macro this stuff, but I'm too lazy
        match self.0 {
            F::BinBytes => [
                (1.0, BB * STEP_UP_FRAC, "B"),
                (BB, BB.powi(2) * STEP_UP_FRAC, "KiB"),
                (BB.powi(2), BB.powi(3) * STEP_UP_FRAC, "MiB"),
                (BB.powi(3), BB.powi(4) * STEP_UP_FRAC, "GiB"),
                (BB.powi(4), BB.powi(5) * STEP_UP_FRAC, "TiB"),
                (BB.powi(5), f64::MAX, "PiB"),
            ],
            F::BinBits => [
                (1.0 / 8.0, BB / 8.0 * STEP_UP_FRAC, "b"),
                (BB / 8.0, BB.powi(2) / 8.0 * STEP_UP_FRAC, "Kib"),
                (BB.powi(2) / 8.0, BB.powi(3) / 8.0 * STEP_UP_FRAC, "Mib"),
                (BB.powi(3) / 8.0, BB.powi(4) / 8.0 * STEP_UP_FRAC, "Gib"),
                (BB.powi(4) / 8.0, BB.powi(5) / 8.0 * STEP_UP_FRAC, "Tib"),
                (BB.powi(5) / 8.0, f64::MAX, "Pib"),
            ],
            F::SiBytes => [
                (1.0, 1e3 * STEP_UP_FRAC, "B"),
                (1e3, 1e6 * STEP_UP_FRAC, "kB"),
                (1e6, 1e9 * STEP_UP_FRAC, "MB"),
                (1e9, 1e12 * STEP_UP_FRAC, "GB"),
                (1e12, 1e15 * STEP_UP_FRAC, "TB"),
                (1e15, f64::MAX, "PB"),
            ],
            F::SiBits => [
                (1.0 / 8.0, 1e3 / 8.0 * STEP_UP_FRAC, "b"),
                (1e3 / 8.0, 1e6 / 8.0 * STEP_UP_FRAC, "kb"),
                (1e6 / 8.0, 1e9 / 8.0 * STEP_UP_FRAC, "Mb"),
                (1e9 / 8.0, 1e12 / 8.0 * STEP_UP_FRAC, "Gb"),
                (1e12 / 8.0, 1e15 / 8.0 * STEP_UP_FRAC, "Tb"),
                (1e15 / 8.0, f64::MAX, "Pb"),
            ],
        }
    }

    /// Select a unit for a given value, returning its divisor and suffix.
    fn get_unit_for(&self, bytes: f64) -> (f64, &'static str) {
        let Some((div, _, suffix)) = self
            .steps()
            .into_iter()
            .find(|&(_, bound, _)| bound >= bytes)
        else {
            panic!("Cannot select an appropriate unit for {bytes:.2}B.")
        };

        (div, suffix)
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
