# Keybswitch

Keybswitch is a Rust application designed to automatically switch keyboard layouts when specific USB keyboards are connected or disconnected. It uses the `udev` library to monitor USB events and `setxkbmap` to change the keyboard layout.

## Features

- Automatically switch keyboard layout when specific keyboards are connected.
- Restore the original layout when the keyboard is disconnected.
- Configurable via a YAML file stored in the XDG configuration directory.

_this is working on nixos, the only system where it was tested, im running gnome/x11_

## Requirements

- Rust (for building the application)
- `setxkbmap` (for changing keyboard layouts)
- A Unix-like operating system with udev support (Linux)

## Installation

1. **Clone the Repository**

   ```sh
   git clone https://github.com/PaysanCorrezien/keybswitch
   cd keybswitch
   ```

2. **Build the Application**

   ```sh
   cargo build --release
   ```

3. **Create the Configuration File**

   Create a `config.yaml` file in the XDG config directory, which is typically `$HOME/.config/keybswitch/`.

   ```sh
   mkdir -p $HOME/.config/keybswitch
   nano $HOME/.config/keybswitch/config.yaml
   ```

   Example `config.yaml`(mine for the test):

   ```yaml
   layout_connected: "us"
   variant_connected: "altgr-intl"
   layout_disconnected: "fr"
   variant_disconnected: ""
   keyboards:
     - name: "Elora Keyboard"
       vendor_id: "8d1d"
       model_id: "9d9d"
     - name: "Cocot46plus"
       vendor_id: "1727"
       model_id: "0003"
   ```

## Configuration

The configuration file `config.yaml` contains the following fields:

- `layout_connected`: The keyboard layout to switch to when a specified keyboard is connected.
- `variant_connected`: The variant of the layout to switch to when a specified keyboard is connected.
- `layout_disconnected`: The keyboard layout to switch to when no specified keyboards are connected.
- `variant_disconnected`: The variant of the layout to switch to when no specified keyboards are connected.
- `keyboards`: A list of keyboards to monitor. Each keyboard has:
  - `name`: A friendly name for the keyboard.
  - `vendor_id`: The vendor ID of the keyboard.
  - `model_id`: The model ID of the keyboard.

## Example

When the Elora Keyboard (vendor ID `8d1d` and model ID `9d9d`) is connected, the keyboard layout will switch to US with the `altgr-intl` variant. When the keyboard is disconnected, the layout will switch back to French.

## License

This project is licensed under the MIT License.
