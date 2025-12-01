# TUI Downloader

A modern, beautiful terminal-based download manager with support for HTTP/HTTPS, FTP, BitTorrent, and Metalink downloads, powered by aria2c.

## Features

- 🚀 **Multi-protocol support** - HTTP, HTTPS, FTP, BitTorrent (magnet links & .torrent files), and Metalink
- 🎨 **Modern UI** - Beautiful floating design with rounded borders and appealing color palette
- ⚡ **Automatic aria2c management** - Spawns and manages aria2c daemon automatically
- 📊 **Real-time monitoring** - Live progress tracking with separate download/upload speed graphs
- 🎯 **Smart detection** - Color-coded speed indicators (warning for low speeds)
- 🔄 **Full control** - Pause, resume, delete from list, or delete files from disk
- 📑 **Organized tabs** - Active, Queue, and Completed downloads
- 💾 **Smart defaults** - Auto-saves to ~/Downloads
- 🛡️ **Safety features** - Quit confirmation and terminal size warnings
- ✨ **User-friendly** - Input validation, status messages, and intuitive shortcuts

## Prerequisites

**aria2c** must be installed on your system:

```bash
# Ubuntu/Debian
sudo apt install aria2

# Fedora
sudo dnf install aria2

# Arch Linux
sudo pacman -S aria2

# macOS
brew install aria2

# Windows (Chocolatey)
choco install aria2
```

## Installation

```bash
git clone https://github.com/yourusername/tui-downloader.git
cd tui-downloader
cargo build --release
```

Binary will be at `target/release/tui-downloader`

## Usage

```bash
cargo run
# or
./target/release/tui-downloader
```

## Keyboard Shortcuts

### Normal Mode
- **`i`** - Add new download
- **`Space/p`** - Pause/Resume
- **`d`** - Delete from list
- **`Shift+Delete`** - Delete file from disk
- **`x`** - Purge completed downloads
- **`1/2/3`** - Switch tabs
- **`↑↓` or `j/k`** - Navigate
- **`q`** - Quit (with confirmation)

### Input Mode
- **`Enter`** - Submit URL
- **`Esc`** - Cancel
- **`Backspace`** - Delete character
- **`Ctrl+V`** - Paste

## Supported Formats

- **URLs**: `https://example.com/file.zip`
- **Magnet links**: `magnet:?xt=urn:btih:...`
- **Torrent files**: `/path/to/file.torrent`
- **Metalink**: `/path/to/file.metalink` or `.meta4`

## Features Highlights

### Split Speed Graphs
- Separate download (↓) and upload (↑) speed visualization
- Color-coded warnings for low speeds (<100 KB/s download, <50 KB/s upload)
- Peak speed indicators
- Real-time sparkline graphs

### Safety Features
- **Quit confirmation** - Warns about active downloads before quitting
- **Terminal size check** - Alerts if terminal is too small (min 100x24)
- **File deletion protection** - Separate shortcuts for list removal vs file deletion

### Modern UI Design
- Rounded borders throughout
- Soft, appealing color palette (RGB pastels)
- Consistent 2-space padding
- Progress bars with labels
- Error messages with context
- Long URL truncation for performance

## Download Types

### HTTP/HTTPS/FTP
Multi-connection downloads for maximum speed.

### BitTorrent
- DHT enabled
- Peer exchange
- Local peer discovery
- Supports both magnet links and .torrent files

### Metalink
Advanced download management with checksums and mirrors.

## Configuration

Default aria2c settings:
- Download directory: `~/Downloads`
- Max connections: 16 per server
- Max concurrent downloads: 5
- BT max peers: 50
- Continues partial downloads

## Troubleshooting

### aria2c not found
Ensure aria2c is installed and in your PATH:
```bash
aria2c --version
```

### Port 6800 in use
Another application is using the default aria2 RPC port. Close it or change the port.

### Terminal too small
Minimum size required: 100 columns × 24 rows. Resize your terminal window.

## License

MIT License - See LICENSE file for details.

## Acknowledgments

- [Ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI framework
- [aria2](https://aria2.github.io/) - Download utility