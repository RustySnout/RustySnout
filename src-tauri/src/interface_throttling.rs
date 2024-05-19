
use std::process::Command;

//sudo tc qdisc add dev $interface root handle 1: htb default 10
//
//sudo tc class add dev $interface parent 1:0 classid 1:10 htb rate $bandwidth burst $burst
//
//sudo tc qdisc add dev $interface parent 1:10 handle 10:0 netem #other stuff goes here

pub fn interface_throttling( interface: &str, bandwidth_limit: u32) {

remove_interface_throttling(interface);

let mut output = Command::new("sudo")
        .arg("tc")
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

    println!("Output: {:?}", output);

    output = Command::new("sudo")
        .arg("tc")
        .arg("class")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("parent")
        .arg("1:0")
        .arg("classid")
        .arg("1:10")
        .arg("htb")
        .arg("rate")
        .arg(format!("{}kbit", bandwidth_limit))
        .output()
        .expect("failed to execute process");

    println!("Output: {:?}", output);

    output = Command::new("sudo")
        .arg("tc")
        .arg("qdisc")
        .arg("add")
        .arg("dev")
        .arg(interface)
        .arg("parent")
        .arg("1:10")
        .arg("handle")
        .arg("10:0")
        .arg("netem")
        .output()
        .expect("failed to execute process");

    println!("Output: {:?}", output);

}

pub fn remove_interface_throttling(interface: &str) {
    let output = Command::new("sudo")
        .arg("tc")
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


// launch app with trickle
pub fn launch_throttle_app(up_limit: u32, down_limit: u32, app: String) {
    let mut output = Command::new("trickle")
        .arg("-s")
        .arg("-d")
        .arg(format!("{}", down_limit))
        .arg("-u")
        .arg(format!("{}", up_limit))
        .arg(app)
        .spawn()
        .expect("failed to execute process")
        .wait_with_output()
        .expect("failed to wait on child");

    println!("Output: {:?}", output);
}

