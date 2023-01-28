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
use eyre::{eyre, Result, WrapErr};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};
use std::collections::HashMap;
use std::env;

pub fn current_ip(host: &str) -> Result<String> {
    let ips = dns_lookup::lookup_host(host).wrap_err(format!("DNS lookup failed on {host}"))?;
    let ip = ips
        .first()
        .ok_or(eyre!("No DNS address entry for {}", host))?;
    Ok(ip.to_string())
}

pub fn send_initial_notification(state: &HashMap<String, String>, restart: bool) -> Result<()> {
    let action = if restart {
        "has been restarted"
    } else {
        "has started"
    };
    let subject = format!("Dynamic DNS monitoring {action}");
    let mut body = vec![format!(
        "Dynamic DNS monitoring {action} on the following hosts:",
    )];
    for (host, addr) in state {
        body.push(format!("-- Host: {host}, Current address: {addr}"))
    }
    body.push(String::from(
        "You will be notified if any of these addresses change.",
    ));
    send_notification(subject, body)
}

pub fn send_change_notification(name: &str, old_address: &str, new_address: &str) -> Result<()> {
    let subject = format!("DNS change for {name}");
    let body = vec![
        format!("The IP address of {name} has changed."),
        format!("-- The old address was: {old_address}."),
        format!("-- The new address is: {new_address}."),
        String::from("You must reconfigure the VPN tunnel."),
    ];
    send_notification(subject, body)
}

pub fn send_notification(subject: String, body: Vec<String>) -> Result<()> {
    let from_address = env::var("DDNS_FROM_ADDRESS")
        .wrap_err("No DDNS_FROM_ADDRESS value found in environment")?;
    let from_password = env::var("DDNS_FROM_PASSWORD")
        .wrap_err("No DDNS_FROM_PASSWORD value found in environment")?;
    let to_address =
        env::var("DDNS_TO_ADDRESS").wrap_err("No DDNS_TO_ADDRESS value found in environment")?;
    let email = Message::builder()
        .from(from_address.parse().unwrap())
        .to(to_address.parse().unwrap())
        .subject(subject)
        .body(body.join("\n"))
        .wrap_err("E-mail message creation failed.")?;
    let creds = Credentials::new(from_address, from_password);
    let mailer = SmtpTransport::relay("smtp.gmail.com")
        .expect("Couldn't lookup smtp.gmail.com")
        .credentials(creds)
        .build();
    let _response = mailer.send(&email).wrap_err("E-mail send failed")?;
    Ok(())
}

pub fn initialize_state() -> Result<HashMap<String, String>> {
    let mut host_list: Vec<(String, String)> = vec![];
    for i in 1..100 {
        let name = format!("DDNS_HOST_{i}");
        match env::var(&name) {
            Ok(host) if !host.is_empty() => {
                let ip = current_ip(&host)?;
                println!(
                    "{}: Initial address for {host} is {ip}",
                    Local::now().to_rfc2822()
                );
                host_list.push((host.clone(), current_ip(&host)?));
            }
            _ => break,
        }
    }
    if host_list.is_empty() {
        return Err(eyre!("No DDNS_HOST_1 value found in environment"));
    }
    Ok(host_list.into_iter().collect())
}

pub fn monitor_state(state: &mut HashMap<String, String>) -> Result<u32> {
    let mut change_count = 0;
    for (name, last_val) in state.iter_mut() {
        let cur_val = current_ip(name)?;
        if !cur_val.eq_ignore_ascii_case(last_val) {
            change_count += 1;
            println!(
                "{}: New address for {name} is {cur_val}",
                Local::now().to_rfc2822()
            );
            send_change_notification(name, last_val, &cur_val).wrap_err("Failed to send email")?;
        }
        *last_val = cur_val;
    }
    Ok(change_count)
}

#[cfg(test)]
mod tests {
    use super::{
        current_ip, initialize_state, monitor_state, send_change_notification,
        send_initial_notification,
    };
    use eyre::Result;
    use std::collections::HashMap;
    use std::env;

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
        let from_address =
            env::var("DDNS_FROM_ADDRESS").expect("No DDNS_FROM_ADDRESS in environment");
        let to_address = env::var("DDNS_TO_ADDRESS").expect("No DDNS_TO_ADDRESS in environment");
        send_change_notification("Some host", "old address", "new address")
            .expect("Failed to send email notification of address change");
        println!("Notification sent from {from_address} to {to_address}");
    }

    fn initialize_state_for_tests() -> Result<HashMap<String, String>> {
        env::set_var("DDNS_HOST_1", "clickonetwo.io");
        env::set_var("DDNS_HOST_2", "localhost");
        env::set_var("DDNS_HOST_3", "");
        initialize_state()
    }

    #[test]
    fn test_initialize_state() {
        let state = initialize_state_for_tests().expect("Couldn't initialize state for tests?");
        assert_eq!(state.len(), 2);
        send_initial_notification(&state, false)
            .expect("Failed to send initial monitor notification");
        send_initial_notification(&state, true)
            .expect("Failed to send restart monitor notification");
    }

    #[test]
    fn test_monitor_state_initial() {
        let mut state = initialize_state_for_tests().expect("Couldn't initialize state for tests?");
        assert_eq!(monitor_state(&mut state).expect("Monitor state failed"), 0);
    }

    #[test]
    fn test_monitor_state_subsequent() {
        let mut state = initialize_state_for_tests().expect("Couldn't initialize state for tests?");
        *state.get_mut("localhost").unwrap() = "incorrect".to_string();
        assert_eq!(monitor_state(&mut state).expect("Monitor state failed"), 1);
    }
}
