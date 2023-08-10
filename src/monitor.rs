/*
MIT License

Copyright (c) 2023 Daniel Brotsky

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
 */
use chrono::Local;
use eyre::{Report, Result, WrapErr};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use crate::Configuration;

use super::{current_ip, State};

pub fn send_initial_notification(config: &Configuration) -> Result<()> {
    let subject = format!("Dynamic DNS monitoring status");
    let mut body = vec![];
    let hostname = gethostname::gethostname().to_string_lossy().to_string();
    let first = config.last_lookup <= 0;
    let action = if first { "starting" } else { "restarting" };
    body.push(format!(
        "Dynamic DNS monitoring from {hostname} is {action} for the following hosts:"
    ));
    let address_type = if first { "Initial" } else { "Last known" };
    for (host, addr) in config.state.iter() {
        body.push(format!("-- Host: {host}, {address_type} address: {addr}"))
    }
    body.push(String::from(
        "You will be notified if any of these addresses change.",
    ));
    send_notification(config, subject, body)
}

pub fn send_change_notification(
    config: &Configuration,
    name: &str,
    old_address: &str,
    new_address: &str,
) -> Result<()> {
    let subject = format!("DNS change for {name}");
    let body = vec![
        format!("The IP address of {name} has changed."),
        format!("-- The old IP address was: {old_address}."),
        format!("-- The new IP address is: {new_address}."),
        String::from("You must reconfigure any services that had the old IP address."),
    ];
    send_notification(config, subject, body)
}

pub fn send_error_notification(config: &Configuration, err: Report) -> Result<()> {
    let subject = format!("DNS monitoring temporary failure");
    let body = vec![
        format!("DNS monitoring reported an error: {err}"),
        format!("A retry will be performed on the normal schedule."),
    ];
    send_notification(config, subject, body)
}

pub fn send_notification(config: &Configuration, subject: String, body: Vec<String>) -> Result<()> {
    let mut builder = Message::builder();
    let from = config.from_address.as_str();
    builder = builder.from(
        from.parse()
            .wrap_err(format!("Illegal from address: {from}"))?,
    );
    for to in config.to_addresses.iter() {
        builder = builder.to(to.parse().wrap_err(format!("Illegal to address: {to}"))?)
    }
    builder = builder.subject(subject);
    let email = builder
        .body(body.join("\n"))
        .wrap_err("E-mail message creation failed.")?;
    let password = config.password()?;
    let creds = Credentials::new(from.to_string(), password.to_string());
    let server = config.from_server.as_str();
    let mailer = SmtpTransport::relay(server)
        .wrap_err(format!("Couldn't lookup {server}"))?
        .credentials(creds)
        .build();
    let _response = mailer.send(&email).wrap_err("E-mail send failed")?;
    Ok(())
}

pub fn initialize_state(config: &Configuration) -> Result<()> {
    let timestamp = Local::now().to_rfc2822();
    println!("{timestamp}: Initializing state monitoring...");
    for (host, ip) in config.state.iter() {
        let timestamp = Local::now().to_rfc2822();
        println!("{timestamp}: The remembered address for {host} is {ip}",);
    }
    send_initial_notification(config)
}

pub fn monitor_once(config: &mut Configuration) -> Result<u32> {
    let mut change_count = 0;
    let mut new_state = State::new();
    for (name, old_address) in config.state.iter() {
        let new_address = current_ip(name)?;
        new_state.insert(name.to_string(), new_address.to_string());
        if !new_address.eq_ignore_ascii_case(old_address) {
            change_count += 1;
            let time = Local::now().to_rfc2822();
            println!("{time}: New address for {name} is {new_address} (was {old_address})");
            send_change_notification(config, name, old_address, &new_address)
                .wrap_err("Failed to send email")?;
        }
    }
    if change_count == 0 {
        let time = Local::now().to_rfc2822();
        println!("{time}: No address changes");
    }
    config.last_lookup = Local::now().timestamp_millis();
    config.state = new_state;
    Ok(change_count)
}

pub fn monitor_loop(config: &mut Configuration, interval_secs: u64) -> ! {
    loop {
        if let Err(err) = monitor_once(config) {
            let timestamp = Local::now().to_rfc2822();
            println!("{timestamp}: Monitor failure: {err}");
            if let Err(err) = send_error_notification(config, err) {
                println!("{timestamp}: Couldn't send error notification: {err}")
            }
        }
        std::thread::sleep(std::time::Duration::from_secs(interval_secs));
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use crate::Configuration;

    use super::{current_ip, initialize_state, monitor_once, send_change_notification};

    #[test]
    fn test_lookup() {
        let cname_ip = current_ip("www.clickonetwo.io")
            .expect("Dynamic DNS lookup failed for CNAME www.clickonetwo.io");
        println!("www.clickonetwo.io IP is: {cname_ip}");
        let a_ip =
            current_ip("clickonetwo.io").expect("Dynamic DNS lookup failed for A clickonetwo.io");
        println!("clickonetwo.io IP is: {a_ip}");
        assert_eq!(cname_ip, a_ip, "CNAME and A record don't match")
    }

    #[test]
    fn test_change_notification() {
        let config = Configuration::new_from_environment(false);
        send_change_notification(&config, "Some host", "old", "new")
            .expect("Failed to send email notification of address change");
    }

    fn get_test_config(is_first_time: bool) -> Configuration {
        env::set_var("DDNS_HOST_1", "clickonetwo.io");
        env::set_var("DDNS_HOST_2", "localhost");
        env::set_var("DDNS_HOST_3", "");
        Configuration::new_from_environment(is_first_time)
    }

    #[test]
    fn test_initialize_state() {
        let config = get_test_config(true);
        assert_eq!(config.state.len(), 2);
        let config = get_test_config(false);
        initialize_state(&config).expect("Failed to initialize state");
    }

    #[test]
    fn test_monitor_state_initial() {
        let mut config = get_test_config(true);
        assert_eq!(monitor_once(&mut config).expect("Monitor state failed"), 0);
    }

    #[test]
    fn test_monitor_state_subsequent() {
        let mut config = get_test_config(false);
        *config.state.get_mut("localhost").unwrap() = "incorrect".to_string();
        assert_eq!(monitor_once(&mut config).expect("Monitor state failed"), 1);
    }
}
