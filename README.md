# CUMD (Computer Use MCP Daemon)

A secure, native OS background service written in **Rust** that exposes a Model Context Protocol (MCP) interface, allowing an Operating Agent (OA) to navigate and control traditional desktop applications (such as Microsoft Word, Excel, Apple Finder, Dolphin, and Calculator) outside constrained web browser sandboxes.

Designed for cross-platform operation across **macOS** (Quartz/ScreenCaptureKit) and **Linux on Wayland** (PipeWire/libei/AT-SPI2).

---

## Key Features

- **Multi-Transport Server**:
  - `stdio`: Local subagent direct execution.
  - `unix_socket`: Local IPC socket with `0600` permissions.
  - `network`: HTTP/SSE transport binding to a local network interface with Bearer token authentication.
- **Privacy-Preserving Window Isolation**:
  - Automatically isolates and crops screenshots strictly to the active application window. Surrounding displays, background apps, taskbars, and notifications are completely excluded.
- **Security Policy Engine**:
  - Application whitelisting (`allowed_apps`) and blacklisting (`blocked_apps`).
  - Gated input injection: rejects clicks and keystrokes if the focused window is not in the whitelist.
- **Cross-Platform OS Engines**:
  - **macOS**: Native Quartz Event Services (`CGEventPost`), `ScreenCaptureKit` / `CGWindowListCreateImage`, and `AXUIElement` accessibility.
  - **Linux Wayland**: XDG Desktop Portals (`ScreenCast` via PipeWire and `RemoteDesktop` via `libei` / `/dev/uinput`), `AT-SPI2` D-Bus accessibility, and automatic `ufw`/`firewalld` port management.

---

## MCP Tools Provided

1. `desktop_capture_screen`: Captures a window-isolated screenshot of the target or active app.
2. `desktop_mouse_action`: Moves, clicks, double clicks, right clicks, drags, and scrolls with coordinate bounds checks.
3. `desktop_keyboard_action`: Types Unicode text, sends virtual keycodes, or fires hotkeys (`Cmd+B`, `Ctrl+S`, etc.).
4. `desktop_inspect_ui`: Introspects the accessibility tree (roles, titles, values, bounds).
5. `desktop_manage_app`: Launches or brings an approved application to the front.
6. `desktop_doctor`: Diagnostic health check verifying OS permissions, transports, and active policy rules.

---

## Quickstart

### Build & Run
```bash
# Diagnostic check of permissions
cargo run -- doctor

# Run MCP daemon (defaulting to stdio or config.toml)
cargo run -- run

# Generate default configuration
cargo run -- init-config --path config.toml
```

### Run Test Suite (TDD)
```bash
# Unit & isolation tests
cargo test --test unit_config --test unit_security_policy --test unit_network_auth --test unit_coordinates

# MCP protocol integration tests
cargo test --test integration_mcp

# Stage 1: Native Calculator E2E algebraic calculation
cargo test --test e2e_stage1_calculator -- --nocapture

# Stage 2: Rich document authoring E2E test
cargo test --test e2e_stage2_document -- --nocapture
```
