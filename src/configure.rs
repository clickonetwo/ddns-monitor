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
use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use eyre::{ContextCompat, Result, WrapErr};
use lettre::{transport::smtp::authentication::Credentials, Address, SmtpTransport};
use magic_crypt::MagicCryptTrait;

use serde::{Deserialize, Serialize};

use super::current_ip;

pub type State = HashMap<String, String>;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub state: State,
    pub to_addresses: Vec<String>,
    pub from_server: String,
    pub from_address: String,
    pub encrypted_password: String,
}

fn config_path(name: Option<&str>) -> Result<PathBuf> {
    let project_dirs = directories::ProjectDirs::from("io", "clickonetwo", "ddns-monitor")
        .wrap_err("Can't find project directories for ddns-monitor.clickonetwo.io")?;
    let config_dir = project_dirs.config_dir();
    let config_name: &str = name.unwrap_or("config.json");
    Ok(config_dir.join(config_name))
}

impl Configuration {
    pub fn new_from_config_file(name: Option<&str>) -> Result<Self> {
        let path = config_path(name)?;
        let config_text = std::fs::read_to_string(path)?;
        let config: Configuration = serde_json::from_str(&config_text)?;
        Ok(config)
    }

    fn save_to_config_file(&self, name: Option<&str>) -> Result<PathBuf> {
        let path = config_path(name)?;
        let config_text =
            serde_json::to_string(self).wrap_err("Configuration cannot be serialized")?;
        let mut file = File::create(path).wrap_err(format!(
            "Config file ({}) cannot be created",
            path.display()
        ))?;
        file.write_all(config_text.as_bytes())
            .wrap_err(format!("Config file ({}) cannot be written", path.display))?;
        Ok(path.clone())
    }

    pub fn new_from_interview() -> Result<Self> {
        let mut new = Configuration::default();
        new.update_from_interview()?;
        Ok(new)
    }

    pub fn update_from_interview(&mut self) -> Result<()> {
        self.interview_from()?;
        self.interview_to_addresses()?;
        self.interview_state()?;
        Ok(())
    }

    fn interview_from(&mut self) -> Result<()> {
        eprintln!("Sending notifications requires an SMTP server, email account, and password.");
        let mut server = if self.from_server.is_empty() {
            "smtp.gmail.com".to_string()
        } else {
            self.from_server.clone()
        };
        let mut account = self.from_address.clone();
        let mut password = if self.encrypted_password.is_empty() {
            String::new()
        } else {
            decrypt_password(self.encrypted_password.as_str())?
        };
        loop {
            server = dialoguer::Input::new()
                .with_prompt("Sending SMTP server")
                .with_initial_text(&server)
                .allow_empty(false)
                .validate_with(|host: &String| -> std::result::Result<(), String> {
                    if current_ip(host).is_ok() {
                        Ok(())
                    } else {
                        Err(format!("{host} is not a valid hostname"))
                    }
                })
                .interact()
                .wrap_err("Input error")?;
            account = dialoguer::Input::new()
                .with_prompt("Sender account email")
                .with_initial_text(&account)
                .allow_empty(false)
                .validate_with(|email: &String| -> std::result::Result<(), String> {
                    if email.parse::<Address>().is_ok() {
                        Ok(())
                    } else {
                        Err(format!("{email} is not a valid email"))
                    }
                })
                .interact()
                .wrap_err("Input error")?;
            password = dialoguer::Input::new()
                .with_prompt("Sender account password")
                .allow_empty(false)
                .validate_with(|pw: &String| -> std::result::Result<(), String> {
                    if pw.eq(pw.trim()) {
                        Ok(())
                    } else {
                        Err(format!("'{pw}' cannot have leading or trailing spaces"))
                    }
                })
                .interact()
                .wrap_err("Input error")?;
            let creds = Credentials::new(account, password);
            let mailer = SmtpTransport::relay(&server)
                .wrap_err(format!("Couldn't lookup {server}"))?
                .credentials(creds)
                .build();
            if mailer.test_connection()? {
                break;
            } else {
                eprintln!("Couldn't connect to that server with those credentials")
            }
        }
        self.from_server = server;
        self.from_address = account;
        self.encrypted_password = encrypt_password(&password)?;
        Ok(())
    }

    fn interview_to_addresses(&mut self) -> Result<()> {
        let mut old_emails = self.to_addresses.iter();
        let mut new_emails = vec![];
        if self.to_addresses.is_empty() {
            eprintln!("Please specify emails to notify, one per line.");
            eprintln!("When done, enter just a dot ('.') alone on the line.");
        } else {
            eprintln!("Please update emails to notify, one per line.");
            eprintln!("To remove an existing name, erase it.");
            eprintln!("When done, enter a dot ('.') alone on the line.");
        }
        loop {
            let prompt = old_emails.next().unwrap_or_default();
            let name: String = dialoguer::Input::new()
                .with_prompt(format!("Email #{}", new_emails.len() + 1))
                .with_initial_text(prompt)
                .allow_empty(true)
                .interact()
                .wrap_err("Input error")?;
            let name = name.trim().to_string();
            if name.is_empty() {
                if !prompt.is_empty() {
                    eprintln!("{prompt} will not be notified.")
                }
                continue;
            } else if name.eq(".") {
                if new_emails.is_empty() {
                    eprintln!("You must specify at least one email to notify");
                    continue;
                }
                break;
            } else if name.parse::<Address>().is_ok() {
                new_emails.push(name.clone());
                eprintln!("{name} added for notification")
            } else {
                eprintln!("{name} is not a valid email address; try again");
            }
        }
        self.to_addresses = new_emails;
        Ok(())
    }

    fn interview_state(&mut self) -> Result<()> {
        let mut old_names = self.state.keys();
        let mut new_state = State::new();
        if self.state.is_empty() {
            eprintln!("Please specify DNS names to check, one per line.");
            eprintln!("When done, enter just a dot ('.') alone on the line.");
        } else {
            eprintln!("Please update your DNS names to check, one per line.");
            eprintln!("To remove an existing name, erase it.");
            eprintln!("When done, enter a dot ('.') alone on the line.");
        }
        loop {
            let prompt = old_names.next().unwrap_or_default();
            let name: String = dialoguer::Input::new()
                .with_prompt(format!("Host #{}", new_state.len() + 1))
                .with_initial_text(prompt)
                .allow_empty(true)
                .interact()
                .wrap_err("Input error")?;
            let name = name.trim().to_string();
            if name.is_empty() {
                if !prompt.is_empty() {
                    eprintln!("{prompt} will not be monitored.")
                }
                continue;
            } else if name.eq(".") {
                if new_state.is_empty() {
                    eprintln!("You must specify at least one DNS name to monitor");
                    continue;
                }
                break;
            } else if let Ok(ip) = current_ip(&name) {
                new_state.insert(name.clone(), ip);
                eprintln!("{name} added for monitoring with initial IP address {ip}")
            } else {
                eprintln!("{name} is not a hostname or has no DNS entry; try again");
            }
        }
        self.state = new_state;
        Ok(())
    }
}

const FALLBACK_MAC_ADDRESS: &'static str = env!("BUILD_MACHINE_MAC_ADDRESS");

fn encrypt_password(pw: &str) -> Result<String> {
    let key = match mac_address::get_mac_address() {
        Ok(Some(addr)) => addr.to_string(),
        _ => FALLBACK_MAC_ADDRESS.to_string(),
    };
    let mc = magic_crypt::new_magic_crypt!(key, 256);
    let base64 = mc.encrypt_str_to_base64(pw);
    Ok(base64)
}

fn decrypt_password(base64: &str) -> Result<String> {
    let key = match mac_address::get_mac_address() {
        Ok(Some(addr)) => addr.to_string(),
        _ => FALLBACK_MAC_ADDRESS.to_string(),
    };
    let mc = magic_crypt::new_magic_crypt!(key, 256);
    let pw = mc
        .decrypt_base64_to_string(base64)
        .wrap_err("Password decryption failed")?;
    Ok(pw)
}

#[cfg(test)]
mod tests {}
