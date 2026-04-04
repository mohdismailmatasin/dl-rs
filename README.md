# dl-rs

A fast, configurable download manager written in Rust. Wraps [aria2](https://github.com/aria2/aria2) with a clean CLI interface, real-time progress bars, and persistent configuration.

## Features

- **Multi-connection downloads** — configurable per-download connections (default: 16)
- **Batch downloads** — pass multiple URLs or a file containing a list of URLs
- **Torrent file support** — download `.torrent` files directly
- **Progress bars** — real-time speed, ETA, and progress via `indicatif`
- **Persistent config** — settings saved between runs
- **Graceful shutdown** — Ctrl+C cleans up partial downloads and `.aria2` temp files
- **Zero-config start** — works out of the box with sensible defaults

## Prerequisites

- [Rust](https://rustup.rs/) (edition 2021 or later)
- [aria2](https://github.com/aria2/aria2) (`aria2c` must be on your `PATH`)

## Installation

### From source

```bash
git clone https://github.com/mohdismailmatasin/dl-rs.git
cd dl-rs
./install.sh
```

This builds the project in release mode and copies the binary to `~/.local/bin`. Make sure that directory is in your `PATH`.

### Manual build

```bash
cargo build --release
./target/release/dl-rs --help
```

## Uninstall

```bash
./uninstall.sh
```

## Usage

```
dl-rs [OPTIONS] [URLS]...
```

### Arguments

| Argument   | Description              |
| ---------- | ------------------------ |
| `[URLS]...`| One or more URLs, torrent file paths, or a text file containing URLs (one per line) |

### Options

| Flag                         | Description                        |
| ---------------------------- | ---------------------------------- |
| `-c, --connections <NUM>`    | Number of connections per download |
| `-o, --dir <PATH>`           | Output directory                   |
| `-h, --help`                 | Print help                         |

### Examples

**Single file download:**

```bash
dl-rs https://example.com/file.zip
```

**Multiple files:**

```bash
dl-rs https://example.com/a.zip https://example.com/b.zip
```

**Download from a list file:**

```bash
dl-rs urls.txt
```

The list file should contain one URL per line. Lines starting with `#` are treated as comments.

**Torrent file:**

```bash
dl-rs ubuntu-24.04.torrent
```

**Custom connections and output directory:**

```bash
dl-rs -c 8 -o /mnt/data https://example.com/large-file.iso
```

## Configuration

On first run, `dl-rs` creates a config file at:

- **Linux:** `~/.config/dl-rs/settings.conf`

The config uses a simple key-value format:

```ini
# dl-rs configuration
# Download directory
download_dir = /home/user/Downloads
# Number of connections per download
connections = 16
```

| Key             | Default             | Description                        |
| --------------- | ------------------- | ---------------------------------- |
| `download_dir`  | `~/Downloads`       | Default output directory           |
| `connections`   | `16`                | Connections per download           |

Command-line arguments (`-c`, `-o`) override config values for the current run.

## How it works

`dl-rs` is a wrapper around `aria2c`. On each invocation it:

1. Spawns a private `aria2c` instance with an ephemeral RPC port
2. Submits downloads via aria2's JSON-RPC API
3. Polls status and renders live progress bars
4. Cleans up the aria2 process and temporary files on completion or interrupt

## Limitations

### Magnet links — not yet supported

Magnet URI (`magnet:?xt=urn:btih:...`) downloads are **not ready**. Currently only:

- HTTP/HTTPS URLs
- Local `.torrent` files

are supported. Magnet link support is planned for a future release.

## License

See [LICENSE](LICENSE).
