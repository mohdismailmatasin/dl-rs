# dl-rs

A fast, multi-threaded command-line download manager built in Rust, powered by [aria2](https://github.com/aria2/aria2).

## Features

- **Multi-file downloads** — Download multiple URLs simultaneously with individual progress bars
- **Torrent support** — Download `.torrent` files directly
- **Batch downloads** — Pass a text file containing URLs (one per line, `#` for comments)
- **Configurable connections** — Set the number of connections per download (default: 16)
- **Custom download directory** — Override the default save location via CLI or config
- **Persistent configuration** — Settings are saved and reused across sessions
- **Real-time progress** — Color-coded progress bars with live speed indicators
- **Ctrl+C support** — Graceful interruption with cleanup

## Prerequisites

- **Rust** (edition 2021+) — [Install Rust](https://rustup.rs/)
- **aria2** — Must be installed and available in your `PATH`

```bash
# Ubuntu/Debian
sudo apt install aria2

# Arch Linux
sudo pacman -S aria2

# macOS
brew install aria2

# Fedora
sudo dnf install aria2
```

## Installation

### Online Install (from GitHub)

```bash
cargo install --git https://github.com/mohdismailmatasin/dl-rs.git
```

This will compile and install the `dl-rs` binary to your Cargo bin directory (`~/.cargo/bin`).

### Local Install (from source)

```bash
git clone https://github.com/mohdismailmatasin/dl-rs.git
cd dl-rs
cargo install --path .
```

Or build directly:

```bash
cargo build --release
sudo cp target/release/dl-rs /usr/local/bin/
```

## Uninstallation

### Online Uninstall

```bash
cargo uninstall dl-rs
```

### Manual Uninstall

```bash
rm /usr/local/bin/dl-rs
```

To also remove the configuration files:

```bash
rm -rf ~/.config/dl-rs
```

## Usage

### Single file download

```bash
dl-rs https://example.com/file.zip
```

### Multiple file downloads

```bash
dl-rs https://example.com/file1.zip https://example.com/file2.zip
```

### Batch download from file

Create a text file with one URL per line:

```
# My download list
https://example.com/file1.zip
https://example.com/file2.zip
```

```bash
dl-rs links.txt
```

### Custom connections

```bash
dl-rs -c 32 https://example.com/file.zip
```

### Custom download directory

```bash
dl-rs -o /path/to/save https://example.com/file.zip
```

### Combined options

```bash
dl-rs -c 32 -o /tmp/downloads https://example.com/file.zip
```

## Configuration

Configuration is stored at:

- **Linux:** `~/.config/dl-rs/settings.conf`
- **macOS:** `~/Library/Application Support/dl-rs/settings.conf`

### Config file format

```ini
# dl-rs configuration
# Download directory
download_dir = /home/user/Downloads
# Number of connections per download
connections = 16
```

CLI flags (`-c`, `-o`) override the saved configuration for the current session.

## Output

### Single download

```
🔻️ Downloading 1 file(s)...
⟪ + ⟫ https://example.com/file.zip
⟪ = ⟫ 16
⟪ / ⟫ /home/user/Downloads

+ 124.2 KiB/s [████████████████████░░░░░░░░░░░░░░░░░░░░░░░░░] 240.00 KiB/1.00 MiB @ 26.59 KiB/s

✓ Complete!
  Total: 1.00 MiB
  Completed: 1/1
  Time: 15.8s
```

### Multiple downloads

```
🔻️ Downloading 3 file(s)...
⟪ + 1 ⟫ https://example.com/1Mb.dat
⟪ + 2 ⟫ https://example.com/10Mb.dat
⟪ + 3 ⟫ https://example.com/100Mb.dat
⟪ = ⟫ 16
⟪ / ⟫ /home/user/Downloads

+ 1 @ 124.2 KiB/s [██████████▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 240.00 KiB/1.00 MiB @ 26.59 KiB/s
+ 2 @ 658.0 KiB/s [██████▒░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] 1.38 MiB/10.00 MiB @ 110.21 KiB/s
+ 3 @ 14627.1 KiB/s [███████████████▓░░░░░░░░░░░░░░░░░░░░░░░░░░░] 37.20 MiB/100.00 MiB @ 2.44 MiB/s

✓ Complete!
✓ Complete!
✓ Complete!
  Total: 111.00 MiB
  Completed: 3/3
  Time: 27.1s
```

## License

Copyright © 2026 Mohd Ismail Mat Asin. All rights reserved.

This software is proprietary. Unauthorized copying, distribution, or modification is strictly prohibited without explicit permission from the copyright holder.
