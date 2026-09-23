use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use desktop_mcp::config::{Config, TransportMode};
use desktop_mcp::hal::DesktopDriver;
use desktop_mcp::security::policy::PolicyEngine;
use desktop_mcp::server::dispatcher::McpDispatcher;
use desktop_mcp::server::transports::{network, socket, stdio};

#[cfg(target_os = "macos")]
use desktop_mcp::hal::macos::MacosDriver;

#[cfg(target_os = "linux")]
use desktop_mcp::hal::wayland::WaylandDriver;

#[derive(Parser)]
#[command(name = "desktop-mcp-daemon")]
#[command(about = "Native Desktop Computer Use MCP Daemon for Operating Agents (OA)", long_about = None)]
struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the MCP daemon
    Run,
    /// Run diagnostic checks on OS permissions and displays
    Doctor,
    /// Interactive setup & permission verification guide for macOS / Linux
    Setup {
        #[arg(long, default_value_t = false)]
        install_service: bool,
    },
    /// Generate a default config file
    InitConfig {
        #[arg(short, long, default_value = "config.toml")]
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    // Default or user-specified config
    let config = if let Some(ref config_path) = cli.config {
        Config::from_file(config_path)?
    } else if std::path::Path::new("config.toml").exists() {
        Config::from_file("config.toml")?
    } else {
        Config::from_toml(include_str!("../config.example.toml"))?
    };

    #[cfg(target_os = "macos")]
    let driver: Arc<dyn DesktopDriver> = Arc::new(MacosDriver::new());

    #[cfg(target_os = "linux")]
    let driver: Arc<dyn DesktopDriver> = Arc::new(WaylandDriver::new());

    match cli.command.unwrap_or(Commands::Run) {
        Commands::Doctor => {
            println!("Running desktop-mcp-daemon doctor diagnostic checks...\n");
            let report = driver.check_permissions().await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Commands::Setup { install_service } => {
            println!("=========================================================");
            println!("  CUMD (desktop-mcp-daemon) System Permission & Setup    ");
            println!("=========================================================\n");

            #[cfg(target_os = "macos")]
            {
                let mut report = driver.check_permissions().await?;

                // 1. Check Accessibility
                if report.accessibility_granted {
                    println!("[✓] macOS Accessibility: GRANTED");
                } else {
                    println!("[!] macOS Accessibility: MISSING");
                    println!("    To enable synthetic mouse/keyboard input and UI inspection,");
                    println!("    grant Accessibility to this terminal or application.");
                    println!("    Opening System Settings > Privacy & Security > Accessibility...\n");

                    let _ = std::process::Command::new("open")
                        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                        .status();

                    println!("    Waiting for Accessibility permission to be enabled...");
                    for _ in 0..15 {
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                        report = driver.check_permissions().await?;
                        if report.accessibility_granted {
                            break;
                        }
                    }

                    if report.accessibility_granted {
                        println!("[✓] macOS Accessibility: GRANTED and CONFIRMED!\n");
                    } else {
                        println!("[X] Still waiting for Accessibility. Please toggle it ON in System Settings.\n");
                    }
                }

                // 2. Check Screen Recording
                if report.screen_recording_granted {
                    println!("[✓] macOS Screen Recording: GRANTED");
                } else {
                    println!("[!] macOS Screen Recording: MISSING");
                    println!("    Opening System Settings > Privacy & Security > Screen Recording...\n");
                    let _ = std::process::Command::new("open")
                        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture")
                        .status();
                }

                // 3. Optional service installation
                if install_service {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    let launch_agents_dir = PathBuf::from(&home).join("Library/LaunchAgents");
                    let _ = std::fs::create_dir_all(&launch_agents_dir);
                    let plist_dest = launch_agents_dir.join("com.user.desktop-mcp-daemon.plist");
                    let plist_src = include_str!("../launchd/com.user.desktop-mcp-daemon.plist");
                    std::fs::write(&plist_dest, plist_src)?;
                    println!("[✓] LaunchAgent installed to {:?}", plist_dest);
                    println!("    To activate: launchctl load {:?}", plist_dest);
                }
            }

            #[cfg(target_os = "linux")]
            {
                println!("[✓] Verifying Linux Wayland environment...");
                if std::env::var("WAYLAND_DISPLAY").is_ok() {
                    println!("[✓] Wayland compositor detected: {}", std::env::var("WAYLAND_DISPLAY").unwrap());
                } else {
                    println!("[!] Warning: WAYLAND_DISPLAY environment variable not set.");
                }

                if install_service {
                    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    let service_dir = PathBuf::from(&home).join(".config/systemd/user");
                    let _ = std::fs::create_dir_all(&service_dir);
                    let service_dest = service_dir.join("desktop-mcp-daemon.service");
                    let service_src = include_str!("../systemd/desktop-mcp-daemon.service");
                    std::fs::write(&service_dest, service_src)?;
                    println!("[✓] Systemd user service installed to {:?}", service_dest);
                    println!("    To activate: systemctl --user daemon-reload && systemctl --user enable --now desktop-mcp-daemon");
                }
            }

            println!("\n[✓] Setup check complete. You can run the daemon with:\n    desktop-mcp-daemon run\n");
        }
        Commands::InitConfig { path } => {
            let template = include_str!("../config.example.toml");
            std::fs::write(&path, template)?;
            println!("Configuration file written to {:?}", path);
        }
        Commands::Run => {
            let policy = Arc::new(PolicyEngine::new(config.security.clone()));
            let dispatcher = Arc::new(McpDispatcher::new(driver, policy));

            match config.server.transport {
                TransportMode::Stdio => {
                    stdio::run_stdio_transport(dispatcher).await?;
                }
                TransportMode::UnixSocket => {
                    let sock_cfg = config.server.unix_socket.unwrap_or_else(|| {
                        desktop_mcp::config::UnixSocketConfig {
                            path: PathBuf::from("/tmp/desktop-mcp.sock"),
                            chmod: "0600".to_string(),
                        }
                    });
                    println!("desktop-mcp-daemon listening on Unix socket {:?}", sock_cfg.path);
                    socket::run_socket_transport(&sock_cfg.path, dispatcher).await?;
                }
                TransportMode::Network => {
                    let net_cfg = config.server.network.unwrap_or_else(|| {
                        desktop_mcp::config::NetworkConfig {
                            interface: "127.0.0.1".to_string(),
                            port: 8443,
                            protocol: "sse".to_string(),
                            auth_token: String::new(),
                            enable_tls: false,
                            cert_file: None,
                            key_file: None,
                        }
                    });
                    network::run_network_transport(
                        &net_cfg.interface,
                        net_cfg.port,
                        &net_cfg.auth_token,
                        dispatcher,
                    ).await?;
                }
            }
        }
    }

    Ok(())
}
