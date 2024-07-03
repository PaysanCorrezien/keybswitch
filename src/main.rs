use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::os::unix::io::AsRawFd;
use std::thread;
use std::time::Duration;
use udev::{Device, Enumerator, MonitorBuilder};
use libc::{poll, pollfd, POLLIN};
use serde::Deserialize;
use xdg::BaseDirectories;

#[derive(Debug, Deserialize)]
struct KeyboardConfig {
    name: String,
    vendor_id: String,
    model_id: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    layout_connected: String,
    variant_connected: String,
    layout_disconnected: String,
    variant_disconnected: String,
    keyboards: Vec<KeyboardConfig>,
}

struct Keyboard {
    name: String,
    vendor_id: String,
    model_id: String,
}

struct KeyboardSwitcher {
    keyboards: Vec<Keyboard>,
    layout_connected: String,
    variant_connected: String,
    layout_disconnected: String,
    variant_disconnected: String,
    known_devices: HashSet<String>,
    layout_changed: bool,
}

impl KeyboardSwitcher {
    fn new(
        layout_connected: &str,
        variant_connected: &str,
        layout_disconnected: &str,
        variant_disconnected: &str,
    ) -> Self {
        Self {
            keyboards: Vec::new(),
            layout_connected: layout_connected.to_string(),
            variant_connected: variant_connected.to_string(),
            layout_disconnected: layout_disconnected.to_string(),
            variant_disconnected: variant_disconnected.to_string(),
            known_devices: HashSet::new(),
            layout_changed: false,
        }
    }

    fn add_keyboard(&mut self, name: &str, vendor_id: &str, model_id: &str) {
        let keyboard = Keyboard {
            name: name.to_string(),
            vendor_id: vendor_id.to_string(),
            model_id: model_id.to_string(),
        };
        self.keyboards.push(keyboard);
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        println!("Starting USB Keyboard Detection and Layout Switch Application");

        loop {
            // Set up udev monitoring
            let monitor = MonitorBuilder::new()?
                .match_subsystem("usb")?
                .listen()?;
            let monitor_fd = monitor.as_raw_fd();

            // Poll for new events
            println!("Monitoring for USB events...");
            let mut fds = [pollfd {
                fd: monitor_fd,
                events: POLLIN,
                revents: 0,
            }];

            let ret = unsafe { poll(fds.as_mut_ptr(), fds.len() as _, 1000) };
            if ret > 0 {
                if fds[0].revents & POLLIN != 0 {
                    for event in monitor.iter() {
                        let device = event.device();
                        println!("New device event: {:?}", device);
                        self.check_device(&device)?;
                    }
                }
            } else if ret < 0 {
                eprintln!("Poll error: {}", std::io::Error::last_os_error());
                thread::sleep(Duration::from_millis(100));
            }

            // Check if the known device is still connected
            if self.layout_changed && !self.is_any_keyboard_connected()? {
                println!("All known keyboards disconnected (manual check)");
                self.layout_changed = false;
                self.set_keyboard_layout(&self.layout_disconnected, &self.variant_disconnected)?;
            }
        }
    }

    fn check_device(&mut self, device: &Device) -> Result<(), Box<dyn Error>> {
        let vendor_id = device.property_value("ID_VENDOR_ID").and_then(|s| s.to_str());
        let model_id = device.property_value("ID_MODEL_ID").and_then(|s| s.to_str());
        let action = device.action().and_then(|a| a.to_str());

        println!("Device Vendor ID: {:?}", vendor_id);
        println!("Device Model ID: {:?}", model_id);
        println!("Device Action: {:?}", action);

        for keyboard in &self.keyboards {
            if vendor_id == Some(&keyboard.vendor_id) && model_id == Some(&keyboard.model_id) {
                let device_id = format!("{:?}:{:?}", vendor_id, model_id);

                if action == Some("add") {
                    println!("{} connected", keyboard.name);
                    self.known_devices.insert(device_id);
                    // Delay added to test timing issues
                    thread::sleep(Duration::from_secs(2));
                    self.set_keyboard_layout(&self.layout_connected, &self.variant_connected)?;
                    self.layout_changed = true;
                }
            }
        }

        Ok(())
    }

    fn is_any_keyboard_connected(&self) -> Result<bool, Box<dyn Error>> {
        let mut enumerator = Enumerator::new()?;
        enumerator.match_subsystem("usb")?;

        for device in enumerator.scan_devices()? {
            let vendor_id = device.property_value("ID_VENDOR_ID").and_then(|s| s.to_str());
            let model_id = device.property_value("ID_MODEL_ID").and_then(|s| s.to_str());
            for keyboard in &self.keyboards {
                if vendor_id == Some(&keyboard.vendor_id) && model_id == Some(&keyboard.model_id) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn set_keyboard_layout(&self, layout: &str, variant: &str) -> Result<(), Box<dyn Error>> {
        let setxkbmap_path = "setxkbmap";
        println!("Found setxkbmap at: {:?}", setxkbmap_path);

        // Log environment variables
        println!("DISPLAY: {:?}", std::env::var("DISPLAY"));
        println!("XAUTHORITY: {:?}", std::env::var("XAUTHORITY"));

        let output = Command::new(setxkbmap_path)
            .arg(layout)
            .arg(variant)
            .output()?;

        println!("Executed setxkbmap command: {:?}", output);

        if output.status.success() {
            println!("Keyboard layout switched successfully");
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        } else {
            eprintln!(
                "Failed to switch keyboard layout: {:?}",
                output.status.code()
            );
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }
}

fn load_config() -> Result<Config, Box<dyn Error>> {
    let xdg_dirs = BaseDirectories::with_prefix("keybswitch")?;
    let config_path = xdg_dirs.place_config_file("config.yaml")?;

    let config_content = fs::read_to_string(config_path)?;
    let config: Config = serde_yaml::from_str(&config_content)?;

    Ok(config)
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = load_config()?;

    let mut switcher = KeyboardSwitcher::new(
        &config.layout_connected,
        &config.variant_connected,
        &config.layout_disconnected,
        &config.variant_disconnected,
    );

    for keyboard in config.keyboards {
        switcher.add_keyboard(
            &keyboard.name,
            &keyboard.vendor_id,
            &keyboard.model_id,
        );
    }

    switcher.run()
}

