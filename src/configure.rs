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
use eyre::{eyre, ContextCompat, Result, WrapErr};
use std::collections::HashMap;

pub type State = HashMap<String, String>;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = camelCase)]
pub struct Configuration {
    state: State,
    #[]
}

fn config_path<T: AsRef<str>>(name: Option<T>) -> Result<std::path::PathBuf> {
    let project_dirs = directories::ProjectDirs::from("io", "clickonetwo", "ddns-monitor")
        .wrap_err("Can't find project directories for ddns-monitor.clickonetwo.io")?;
    let config_dir = project_dirs.config_dir();
    let config_name: &str = name.unwrap_or("state.json");
    Ok(config_dir.join("state.json"))
}

pub fn load_state<T: AsRef<str>>(name: Option<T>) -> Result<State> {
    let path = config_path(name)?;
    let config_text = std::fs::read_to_string(path)?;
    let state: State = serde_json::from_str(&config_text)?;
    if state.is_empty() {
        return Err(eyre!("No saved hosts to monitor"));
    }
    Ok(state)
}

pub fn configure<T: AsRef<str>>(name: Option<T>) -> Result<()> {
    let path = config_path(name)?;
    let old_state: State = if let Ok(config_text) = std::fs::read_to_string(path) {
        serde_json::from_str(&config_text).unwrap_or_default()
    } else {
        State::new()
    };
    let new_state = configure_state(old_state);
    if new_state.is_empty() {
        return Err(eyre!("You must specify at least DNS name to monitor."));
    }
    Ok(())
}

fn configure_state(old_state: State) -> Result<State> {
    let mut old_names = old_state.keys();
    let mut new_state = State::new();
    if old_state.is_empty() {
        eprintln!("Please specify DNS names to check, one per line.");
        eprintln!("When done, enter just a dot ('.') alone on the line.");
    } else {
        eprintln!("Please update your DNS names to check, one per line.");
        eprintln!("To remove an existing name, change it to a minus sign ('-').");
        eprintln!("When done, enter a dot ('.') alone on the line.");
    }
    loop {
        let prompt = old_names.next().unwrap_or_default();
        let name = dialoguer::Input::new()
            .with_prompt(format!("Host {}", new_state.len() + 1))
            .with_initial_text(prompt)
            .allow_empty(false)
            .interact()
            .wrap_err("Input error")?;
        let name = name.trim();
        if name == "." {
            break;
        } else if name == "-" {
            if !prompt.is_empty() {
                eprintln!("{prompt} will not be monitored.")
            }
            continue;
        } else if let Ok(ip) = super::monitor::current_ip(name) {
            new_state.insert(name.to_string(), ip);
            eprintln!("{name} added for monitoring with initial IP address {ip}")
        } else {
            eprintln!("{name} is not a hostname or has no DNS entry; try again");
        }
    }
    Ok(new_state)
}

#[cfg(test)]
mod tests {}
