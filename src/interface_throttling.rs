
use std::process::Command;

pub fn interface_Throttling( interface: &str, bandwidth_limit: u32) {
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
        .arg("12")
        .spawn()
        .expect("Failed to add qdisc")
        .wait_with_output()
        .expect("Failed to wait on child");

    println!("Output: {:?}", output);

    let output = Command::new("tc")
        .arg("class")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("parent")
        .arg("1:")
        .arg("classid")
        .arg("1:12")
        .arg("htb")
        .arg("rate")
        .arg(format!("{}kbit", bandwidth_limit))
        .spawn()
        .expect("Failed to add class")
        .wait_with_output()
        .expect("Failed to wait on child");

    println!("Output: {:?}", output);
}

pub fn remove_interface_throttling(interface: &str) {
    let output = Command::new("tc")
        .arg("qdisc")
        .arg("del")
        .arg("dev")
        .arg(interface)
        .arg("root")
        .spawn()
        .expect("Failed to delete qdisc")
        .wait_with_output()
        .expect("Failed to wait on child");

    println!("Output: {:?}", output);
}


