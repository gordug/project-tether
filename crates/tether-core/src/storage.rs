use crate::models::{AuthMethod, HostRecord, ProtocolType};
use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Could not determine user config directory")]
    NoConfigDir,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct ConfigManager {
    config_dir: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self, StorageError> {
        let proj = ProjectDirs::from("com", "tether", "tether")
            .ok_or(StorageError::NoConfigDir)?;
        let config_dir = proj.config_dir().to_path_buf();
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }
        Ok(Self { config_dir })
    }

    fn hosts_file_path(&self) -> PathBuf {
        self.config_dir.join("hosts.json")
    }

    pub fn load_hosts(&self) -> Result<Vec<HostRecord>, StorageError> {
        let path = self.hosts_file_path();
        if !path.exists() {
            let default_hosts = Self::sample_hosts();
            self.save_hosts(&default_hosts)?;
            return Ok(default_hosts);
        }

        let content = fs::read_to_string(path)?;
        let hosts: Vec<HostRecord> = serde_json::from_str(&content)?;
        Ok(hosts)
    }

    pub fn save_hosts(&self, hosts: &[HostRecord]) -> Result<(), StorageError> {
        let path = self.hosts_file_path();
        let content = serde_json::to_string_pretty(hosts)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn sample_hosts() -> Vec<HostRecord> {
        vec![
            HostRecord {
                id: "10000000-0000-0000-0000-000000000001".to_string(),
                name: "Homelab Linux Core".to_string(),
                group: "Infrastructure".to_string(),
                protocol: ProtocolType::Ssh,
                host: "192.168.1.100".to_string(),
                port: 22,
                username: "admin".to_string(),
                auth_method: AuthMethod::KeyFile,
                key_path: Some("~/.ssh/id_ed25519".to_string()),
                tags: vec!["linux".into(), "primary".into(), "docker".into()],
                notes: Some("Main homelab server hosting containers and monitoring".into()),
                last_connected: Some("Today, 14:22".into()),
                is_favorite: true,
            },
            HostRecord {
                id: "10000000-0000-0000-0000-000000000002".to_string(),
                name: "Network Switch Core".to_string(),
                group: "Network".to_string(),
                protocol: ProtocolType::Telnet,
                host: "192.168.1.1".to_string(),
                port: 23,
                username: "admin".to_string(),
                auth_method: AuthMethod::Password,
                key_path: None,
                tags: vec!["switch".into(), "cisco".into()],
                notes: Some("Management plane for core 24-port switch".into()),
                last_connected: Some("Yesterday".into()),
                is_favorite: false,
            },
            HostRecord {
                id: "10000000-0000-0000-0000-000000000003".to_string(),
                name: "Storage NAS Cluster".to_string(),
                group: "Storage".to_string(),
                protocol: ProtocolType::Sftp,
                host: "192.168.1.150".to_string(),
                port: 22,
                username: "backup".to_string(),
                auth_method: AuthMethod::Password,
                key_path: None,
                tags: vec!["zfs".into(), "backups".into()],
                notes: Some("High capacity ZFS storage pool with automated snapshots".into()),
                last_connected: Some("3 days ago".into()),
                is_favorite: true,
            },
            HostRecord {
                id: "10000000-0000-0000-0000-000000000004".to_string(),
                name: "Windows Hyper-V Workstation".to_string(),
                group: "Workstations".to_string(),
                protocol: ProtocolType::Rdp,
                host: "192.168.1.200".to_string(),
                port: 3389,
                username: "developer".to_string(),
                auth_method: AuthMethod::Password,
                key_path: None,
                tags: vec!["windows".into(), "gui".into(), "hyperv".into()],
                notes: Some("Remote development workstation with GPU passthrough".into()),
                last_connected: None,
                is_favorite: false,
            },
            HostRecord {
                id: "10000000-0000-0000-0000-000000000005".to_string(),
                name: "Raspberry Pi Headless Display".to_string(),
                group: "IoT & Edge".to_string(),
                protocol: ProtocolType::Vnc,
                host: "192.168.1.88".to_string(),
                port: 5900,
                username: "pi".to_string(),
                auth_method: AuthMethod::Password,
                key_path: None,
                tags: vec!["arm64".into(), "vnc".into()],
                notes: Some("Edge display console running kiosk dashboard".into()),
                last_connected: None,
                is_favorite: false,
            },
        ]
    }
}
