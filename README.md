# Project Tether

**Cross-Platform Remote Management, Terminal & Server Monitoring Workbench**

[![Rust](https://img.shields.io/badge/Rust-2024_Edition-orange.svg)](https://www.rust-lang.org)
[![Slint](https://img.shields.io/badge/Slint-1.18.1-blue.svg)](https://slint.dev)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**Tether** is a high-performance, single-binary, cross-platform remote management workbench built with **Rust** and **Slint UI**. It consolidates SSH, Telnet, SCP/SFTP, FTP/FTPS, VNC, RDP, and real-time server health monitoring into a unified, responsive desktop environment.

---

## Architecture & Project Plan

For the comprehensive system architecture, subsystem strategies, and phased development roadmap, see the master project plan:

👉 **[Master Project Architectural & Implementation Plan](docs/PROJECT_PLAN.md)**

### Current Progress & Roadmap Status

| Phase | Milestone | Status | Key Deliverables |
| :--- | :--- | :---: | :--- |
| **Phase 1** | **Foundation, Storage & UI Shell** | **Completed** | • Multi-crate Cargo workspace (`tether`, `tether-core`, `tether-net`)<br>• Encrypted zero-knowledge vault (Argon2id + ChaCha20-Poly1305 + Zeroize)<br>• Persistent host inventory configuration manager<br>• Modern Slint UI shell with multi-session tabs, search/filter, quick connect, and live metric dials |
| **Phase 2** | **Terminal Engine & Remote Shells** | *Planned* | • Async SSH client integration (`russh`)<br>• VT100/ANSI escape sequence parser & Slint grid viewport<br>• Telnet client with NVT option negotiation |
| **Phase 3** | **Dual-Pane File Transfer Suite** | *Planned* | • SFTP, SCP, and FTP/FTPS protocol engines<br>• Dual-pane file manager with background transfer queue |
| **Phase 4** | **Agentless Host Telemetry** | *Planned* | • Periodic SSH metrics poller (`/proc`, `sysctl`, PowerShell)<br>• ICMP/TCP ping heartbeat sweeper<br>• Dynamic sparkline and gauge visualizers |
| **Phase 5** | **Graphical Remote Desktop** | *Planned* | • VNC / RFB protocol client with `SharedPixelBuffer` rendering<br>• RDP client integration via `ironrdp`<br>• Low-latency input event forwarding |
| **Phase 6** | **Packaging, Hardening & Release** | *Planned* | • Cross-platform distribution (Linux `.deb`/Flatpak/AppImage, macOS `.dmg`, Windows `.msi`)<br>• Theme switching and security audit |

---

## Key Features

- **Unified Host Inventory**: Organize hosts with tags, groups, protocols, and favorite status, with instant fuzzy-style search and category filtering.
- **Zero-Knowledge Encrypted Vault**: Master password key derivation via Argon2id, AEAD storage via ChaCha20-Poly1305, and automatic memory hygiene (`ZeroizeOnDrop`).
- **Multi-Protocol Workbench**: Native support planned for SSH, Telnet, SFTP, SCP, FTP/FTPS, VNC, and RDP.
- **Fast, Native Reactive UI**: Declarative UI written in Slint 1.18+ with hardware acceleration across Linux, macOS, and Windows.
- **Tabbed Workspace**: Seamlessly juggle multiple sessions with persistent status indicators and quick-connect capabilities.
- **Live Health Monitoring**: Real-time dials and sparklines tracking CPU, RAM, disk, latency, and load average.

---

## Workspace Structure

```
project-tether/
├── Cargo.toml               # Workspace configuration
├── docs/
│   └── PROJECT_PLAN.md      # Master architectural and implementation roadmap
├── src/
│   └── main.rs              # Desktop application entrypoint & Slint event binding
├── ui/                      # Slint UI definitions
│   ├── app-window.slint     # Root application window shell
│   ├── theme.slint          # Design tokens, color palette & typography
│   ├── models.slint         # UI data structures & global singletons
│   └── components/
│       ├── sidebar.slint    # Host tree, search, filter, and connection cards
│       ├── header.slint     # Global navigation, quick run bar, actions
│       ├── workspace.slint  # Session tabs, terminal viewport, telemetry dashboard
│       └── new_host_dialog.slint # Modal dialog for configuring new hosts
└── crates/
    ├── tether-core/         # Domain models, Argon2id/ChaCha20 vault, config storage
    └── tether-net/          # Protocol descriptors and network adapters
```

---

## Getting Started

### Prerequisites

- **Rust toolchain** (Rust 2024 edition / stable 1.85+ recommended):
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

### Build & Run

1. **Clone the repository:**
   ```bash
   git clone https://github.com/gordug/project-tether.git
   cd project-tether
   ```

2. **Run tests across the workspace:**
   ```bash
   cargo test --workspace
   ```

3. **Launch the application:**
   ```bash
   cargo run
   ```

4. **Build optimized release binary:**
   ```bash
   cargo build --release
   ```

---

## License

This project is licensed under the [MIT License](LICENSE).
