# Vega Hotline Checker

A command-line tool to extract hotline information from Sangoma Vega devices. This tool can connect to one or more Vega devices, download their configuration, and parse it to determine which phone port (subscriber) is configured to automatically dial a specific destination number (hotline).

The output can be saved as an Excel spreadsheet or a JSON file.

## Installation

There are two ways to use this tool:

### From a Release (Recommended)

You can download the latest pre-compiled binaries for Linux and Windows from the [Releases](https://github.com/Fourteen-IP/vega-hotline/releases) page. This is the easiest way to get started.

### From Source

If you want to build and run the tool from source, you'll need to have Rust and Cargo installed. See [https://rustup.rs/](https://rustup.rs/) for installation instructions.

Then clone the repository and build:

```bash
git clone https://github.com/Fourteen-IP/vega-hotline.git
cd vega-hotline
cargo build --release
```

The compiled binary will be in `target/release/hotline`.

---

## Usage

You can run the tool either with the pre-compiled binary or by building from source.

### Scan a Single Device

```bash
hotline scan -s <VEGA_IP_ADDRESS> -u <USERNAME> -p <PASSWORD> -x output.xlsx
```

This connects to the specified Vega device and exports the dial plan data to an Excel file (`output.xlsx` by default).

### Scan an IP Range

```bash
hotline scan -s <START_IP> -e <END_IP> -u <USERNAME> -p <PASSWORD> -x output.xlsx -j output.json
```

Scans all devices in the given IP range and exports results to Excel and JSON.

### Parse a Local Configuration File

```bash
hotline config -c /path/to/config.txt -x output.xlsx -j output.json
```

Parses a local Vega config file instead of connecting to a device.

---

## Command-Line Arguments

```text
Usage: hotline <COMMAND> [OPTIONS]

Commands:

  scan       Scan Vega device(s) over IP and fetch configuration
  config     Parse a local config file instead of querying a device

Options:

  -v, --verbose         Increase logging verbosity (use multiple times for more detail)
  -q, --quiet           Suppress non-critical log messages

Scan Options:

  -s, --start-ip <IP>   Start IP address (required)
  -e, --end-ip <IP>     End IP address (optional, requires start-ip)
  -u, --username <STR>  Vega username (required)
  -p, --password <STR>  Vega password (required)
  -x, --excel-file <FILE>  Output Excel file path (optional, default: output.xlsx)
  -j, --json-file <FILE>   Output JSON file path (optional)

Config Options:

  -c, --config-file <FILE>  Path to local config file (required)
  -x, --excel-file <FILE>   Output Excel file path (optional, default: output.xlsx)
  -j, --json-file <FILE>    Output JSON file path (optional)
```

---

## Logging

Control logging verbosity using the `-v` flag:

- No `-v`: warnings and errors only
- `-v`: info level
- `-vv`: debug level
- `-vvv` or more: trace level

Example:

```bash
hotline scan -s 192.168.1.10 -u admin -p secret -vv -x output.xlsx
```