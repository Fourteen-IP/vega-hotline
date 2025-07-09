# Vega Hotline Checker

A command-line tool to extract hotline information from Sangoma Vega devices. This tool can connect to one or more Vega devices, download their configuration, and parse it to determine which phone port (subscriber) is configured to automatically dial a specific destination number (hotline).

The output can be saved as an Excel spreadsheet or a JSON file.

## Installation

There are two ways to use this tool:

### From a Release (Recommended)

You can download the latest pre-compiled binaries for Linux and Windows from the Releases page. This is the easiest way to get started.

### From Source

If you want to run the tool from source, you'll need to:

1.  Clone this repository.
2.  Install the required Python packages using `uv`:

    ```bash
    uv sync
    ```

## Usage

You can run the tool using the Python script (`main.py`) or with the pre-compiled binary (`hotline` for Linux, `hotline.exe` for Windows) available in the project's releases.

### Basic Usage

To connect to a single Vega device and save the output to an Excel file:

**Using Python:**
```bash
python main.py -s <VEGA_IP_ADDRESS> -u <USERNAME> -p <PASSWORD> -x output.xlsx
```

**Using the binary:**
```bash
./hotline -s <VEGA_IP_ADDRESS> -u <USERNAME> -p <PASSWORD> -x output.xlsx
```

### Scan an IP Range

To scan a range of IP addresses:

**Using Python:**
```bash
python main.py -s <START_IP> -e <END_IP> -u <USERNAME> -p <PASSWORD> -x output.xlsx
```

**Using the binary:**
```bash
./hotline -s <START_IP> -e <END_IP> -u <USERNAME> -p <PASSWORD> -x output.xlsx
```

### Use a Local Configuration File

If you have a Vega configuration file (`config.txt`) saved locally, you can use it directly:

**Using Python:**
```bash
python main.py -c /path/to/your/config.txt -x output.xlsx
```

**Using the binary:**
```bash
./hotline -c /path/to/your/config.txt -x output.xlsx
```

### Command-Line Arguments

```
usage: hotline.exe [-h] [-s START_IP] [-e END_IP] -u USERNAME -p PASSWORD [-x TO_EXCEL] [-j JSON] [-q QUIET] [-c]

CLI Tool designed to return hotline information from Sangoma Vegas

optional arguments:
  -h, --help            show this help message and exit
  -s , --start-ip       start IP address
  -e , --end-ip         end IP address
  -v, --verbose         enable verbose logging
  -x , --to-excel       file path to output to an excel spreadsheet
  -j , --json           file path to output to a json file
  -q, --quiet           ignore warnings
  -c , --from-config    file path to use config file instead of ip address

credentials:
  -u , --username       vega Username
  -p , --password       vega Password
```

## Building from Source

This project uses [Nuitka](https://nuitka.net/) to compile the Python script into a standalone executable.

The included GitHub Actions workflow (`.github/workflows/python-app.yml`) is configured to build the executable for Windows and Linux automatically.

To build it manually, you would typically run Nuitka like this:

```bash
python -m nuitka --onefile main.py
```

The compiled executable will be placed in the current directory.
