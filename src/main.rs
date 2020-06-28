use clap::{App, Arg};
use smartplug::plugs::{Kasa, Smartplug};
use std::io;
use std::net::IpAddr;
use std::process::Command;
use std::{thread, time};

fn reboot_modem(plug: &impl Smartplug, grace_period: time::Duration) -> Result<(), io::Error> {
    println!("Cutting modem power.");
    plug.set_power(false)?;
    thread::sleep(time::Duration::from_secs(3));
    println!("Starting modem ...");
    plug.set_power(true)?;
    println!(
        "Waiting {} seconds for the modem to boot.",
        grace_period.as_secs()
    );
    thread::sleep(grace_period);
    Ok(())
}

fn iface_is_up(name: &str) -> Result<bool, io::Error> {
    let raw_output = Command::new("ifconfig").arg(name).output()?;

    if !raw_output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "ifconfig exited with a non-zero exit code. Output: {}",
                String::from_utf8(raw_output.stderr).unwrap()
            ),
        ));
    }

    let output = String::from_utf8(raw_output.stdout).unwrap();

    let prefix = "inet ";

    let ip_index = match output.find(prefix) {
        Some(index) => index + prefix.len(),
        _ => return Ok(false),
    };

    let next_whitespace = match &output[ip_index..].find(' ') {
        Some(index) => index + ip_index,
        _ => return Ok(false),
    };

    let ip_string = &output[ip_index..next_whitespace];

    let ip = match ip_string.parse::<IpAddr>() {
        Ok(ip) => ip,
        _ => return Ok(false),
    };

    println!("{} IP: {}", name, ip);

    // println!("{}", output);

    Ok(false)
}

fn main() {
    let matches = App::new("pfSense Modem Rebooter")
        .author("Åke Amcoff <ake.amcoff@lynx.agency>")
        .about(
            "Automatically reboot a modem with the help of a smart plug when an interface is down.",
        )
        .arg(
            Arg::with_name("iface")
                .short("i")
                .long("iface")
                .value_name("INTERFACE")
                .help("Name of the interface to monitor.")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("plug")
                .short("p")
                .long("plug")
                .value_name("PLUG")
                .help("The IP address of the smart plug.")
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let iface = matches.value_of("iface").unwrap();
    println!("Monitoring interface: {}", iface);

    let plug_ip = matches.value_of("plug").unwrap().parse::<IpAddr>().unwrap();
    println!("Smart plug IP: {}", plug_ip);

    let check_interval = time::Duration::from_secs(5);

    let plug = Kasa::new(plug_ip);

    loop {
        let is_up = iface_is_up(iface).unwrap();

        if !is_up {
            println!("Interface {} detected down.", iface);
            reboot_modem(&plug, time::Duration::from_secs(30)).unwrap()
        }

        thread::sleep(check_interval);
    }
}
