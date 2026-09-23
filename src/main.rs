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
