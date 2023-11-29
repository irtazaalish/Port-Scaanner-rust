# Port Scanner in Rust

A simple multi-threaded port scanner written in Rust.

## Features

- **Multi-threaded Scanning:** Utilizes multiple threads to speed up the port scanning process.
- **IPv4 Support:** Supports scanning IPv4 addresses.
- **Flexible Target Specification:** Scan a single IP address or a range of IP addresses.

## Usage

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) must be installed.

### Building

Clone the repository and navigate to the project directory:

```bash
git clone https://github.com/irtazaalish/Port-Scanner-rust.git
cd Port-Scanner-Rust

Build the project using Cargo:

```bash

cargo build --release

Running

Scan a single IP address:

```bash

cargo run --release -- <IP_ADDRESS>

Scan a range of IP addresses:

```bash

cargo run --release -- <START_IP_ADDRESS>

Example

```bash

cargo run --release -- 192.168.1.1

## Configuration

Threads: The number of threads used for scanning can be configured in the main.rs file.

## TODOs

Improve error handling for more informative error messages.
Add support for IPv6.
Implement a more sophisticated mechanism for handling IP address ranges.
Provide options for customizing scan parameters (timeout, retry attempts, etc.).

## Download Binary

Go to the [Releases](https://github.com/irtazaalish/Port-Scanner-rust/releases/) page on GitHub.
Download the binary for your platform from the latest release.
