// # Define the IP address you want to throttle
// DESTINATION_IP="x.x.x.x"
// 
// # Define the rate limit you want to impose (in this example, 1mbit/s)
// RATE_LIMIT="1mbit"
// 
// # Delete any existing rules for this IP (optional, but ensures no conflicts)
// tc qdisc del dev eth0 root
// 
// # Add a new qdisc (queuing discipline) to the network interface
// tc qdisc add dev eth0 root handle 1: htb default 10
// 
// # Create a class under the root qdisc
// tc class add dev eth0 parent 1: classid 1:1 htb rate $RATE_LIMIT
// 
// # Define a filter for the destination IP address
// tc filter add dev eth0 protocol ip parent 1: prio 1 u32 match ip dst $DESTINATION_IP flowid 1:1

use std::process::Command;

pub fn throttle_ip(ip: &str, rate: &str, interface: &str) {
    
    let output = Command::new("tc")
        .arg("qdisc")
        .arg("del")
        .arg("dev")
        .arg(interface)
        .arg("root")
        .output()
        .expect("failed to execute process");

    let output = Command::new("tc")
        .arg("qdisc")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("root")
        .arg("handle")
        .arg("1:")
        .arg("htb")
        .arg("default")
        .arg("10")
        .output()
        .expect("failed to execute process");

    let output = Command::new("tc")
        .arg("class")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("parent")
        .arg("1:")
        .arg("classid")
        .arg("1:1")
        .arg("htb")
        .arg("rate")
        .arg(rate)
        .output()
        .expect("failed to execute process");

    let output = Command::new("tc")
        .arg("filter")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("protocol")
        .arg("ip")
        .arg("parent")
        .arg("1:")
        .arg("prio")
        .arg("1")
        .arg("u32")
        .arg("match")
        .arg("ip")
        .arg("dst")
        .arg(ip)
        .arg("flowid")
        .arg("1:1")
        .output()
        .expect("failed to execute process");

    println!("Throttling IP: {} at rate: {} on interface: {}", ip, rate, interface);
}