use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProtocolType {
    Ssh,
    Telnet,
    Sftp,
    Scp,
    Ftp,
    Vnc,
    Rdp,
}

impl ProtocolType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Ssh => "SSH",
            Self::Telnet => "Telnet",
            Self::Sftp => "SFTP",
            Self::Scp => "SCP",
            Self::Ftp => "FTP",
            Self::Vnc => "VNC",
            Self::Rdp => "RDP",
        }
    }

    pub fn default_port(&self) -> u16 {
        match self {
            Self::Ssh => 22,
            Self::Telnet => 23,
            Self::Sftp => 22,
            Self::Scp => 22,
            Self::Ftp => 21,
            Self::Vnc => 5900,
            Self::Rdp => 3389,
        }
    }

    pub fn from_str_lenient(s: &str) -> Self {
        match s.to_ascii_uppercase().as_str() {
            "TELNET" => Self::Telnet,
            "SFTP" => Self::Sftp,
            "SCP" => Self::Scp,
            "FTP" => Self::Ftp,
            "VNC" => Self::Vnc,
            "RDP" => Self::Rdp,
            _ => Self::Ssh,
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::Ssh | Self::Telnet => "Terminal",
            Self::Sftp | Self::Scp | Self::Ftp => "File Transfer",
            Self::Vnc | Self::Rdp => "Remote Desktop",
        }
    }
}

impl fmt::Display for ProtocolType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    None,
    Password,
    KeyFile,
    Agent,
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::Password
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostRecord {
    pub id: String,
    pub name: String,
    pub group: String,
    pub protocol: ProtocolType,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth_method: AuthMethod,
    pub key_path: Option<String>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub last_connected: Option<String>,
    pub is_favorite: bool,
}

impl HostRecord {
    pub fn new(
        name: impl Into<String>,
        protocol: ProtocolType,
        host: impl Into<String>,
        username: impl Into<String>,
    ) -> Self {
        let port = protocol.default_port();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            group: "Default".to_string(),
            protocol,
            host: host.into(),
            port,
            username: username.into(),
            auth_method: AuthMethod::Password,
            key_path: None,
            tags: Vec::new(),
            notes: None,
            last_connected: None,
            is_favorite: false,
        }
    }
}
