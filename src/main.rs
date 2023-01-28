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
use ddns_monitor::{initialize_state, monitor_state, send_initial_notification};
use std::env;

fn main() {
    let interval: u64 = env::var("DDNS_INTERVAL_SECONDS")
        .unwrap_or_else(|_| String::from("3600"))
        .parse()
        .unwrap_or(3600);
    let mut state = initialize_state().expect("Initialization error");
    send_initial_notification(&state, false).expect("Initial notification error");
    loop {
        match monitor_state(&mut state) {
            Ok(_) => {
                std::thread::sleep(std::time::Duration::from_secs(interval));
            }
            Err(err) => {
                println!("{}: Monitor failure: {err}", Local::now().to_rfc2822());
                std::thread::sleep(std::time::Duration::from_secs(interval));
                send_initial_notification(&state, true).expect("Restart notification error");
            }
        }
    }
}
