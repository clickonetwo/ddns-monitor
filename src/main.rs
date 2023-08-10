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
use std::env;

use chrono::Local;
use clap::Parser;
use eyre::{Result, WrapErr};

use ddns_monitor::{initialize_state, monitor_loop, Configuration};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
enum Command {
    Configure,
    Monitor,
}

fn main() -> Result<()> {
    let command: Command = Command::parse();
    let result = Configuration::new_from_config_file();
    match command {
        Command::Monitor => match result {
            Ok(mut config) => monitor(&mut config),
            Err(err) => {
                let timestamp = Local::now().to_rfc2822();
                eprintln!("{timestamp}: Fatal error: can't load configuration: {err}");
                std::process::exit(1);
            }
        },
        Command::Configure => {
            let mut config = match result {
                Ok(config) => config,
                Err(_) => Configuration::default(),
            };
            config
                .update_from_interview()
                .wrap_err("Failed to update configuration")?;
            config
                .save_to_config_file()
                .wrap_err("Failed to save configuration")?;
            Ok(())
        }
    }
}

fn monitor(config: &mut Configuration) -> Result<()> {
    // first give the machine time to finish booting, in case we run at startup
    let start_wait: u64 = env::var("DDNS_START_DELAY_SECONDS")
        .unwrap_or_else(|_| String::from("300"))
        .parse()
        .unwrap_or(300);
    if start_wait > 0 {
        println!(
            "{}: DDNS Monitor starting (delay {start_wait} seconds)...",
            Local::now().to_rfc2822()
        );
        std::thread::sleep(std::time::Duration::from_secs(start_wait));
    }
    initialize_state(config).wrap_err("Initialization error")?;
    let interval: u64 = env::var("DDNS_INTERVAL_SECONDS")
        .unwrap_or_else(|_| String::from("3600"))
        .parse()
        .unwrap_or(3600);
    monitor_loop(config, interval);
}
