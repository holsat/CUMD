use std::process::Command;

pub struct FirewallManager;

impl FirewallManager {
    pub fn open_port(port: u16) -> Result<(), String> {
        // 1. Try ufw
        if let Ok(status) = Command::new("ufw").args(["status"]).status() {
            if status.success() {
                let _ = Command::new("ufw")
                    .args(["allow", &format!("{}/tcp", port)])
                    .status();
                return Ok(());
            }
        }

        // 2. Try firewalld
        if let Ok(status) = Command::new("firewall-cmd").args(["--state"]).status() {
            if status.success() {
                let _ = Command::new("firewall-cmd")
                    .args(["--add-port", &format!("{}/tcp", port)])
                    .status();
                return Ok(());
            }
        }

        Ok(())
    }

    pub fn close_port(port: u16) -> Result<(), String> {
        // Try ufw
        let _ = Command::new("ufw")
            .args(["delete", "allow", &format!("{}/tcp", port)])
            .status();

        // Try firewalld
        let _ = Command::new("firewall-cmd")
            .args(["--remove-port", &format!("{}/tcp", port)])
            .status();

        Ok(())
    }
}
