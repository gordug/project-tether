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

    pub fn is_file_transfer(&self) -> bool {
        matches!(self, Self::Sftp | Self::Scp | Self::Ftp)
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Ssh | Self::Telnet)
    }

    pub fn is_desktop(&self) -> bool {
        matches!(self, Self::Vnc | Self::Rdp)
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

/// Represents a file or directory item in local or remote file systems
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size_bytes: u64,
    pub size_formatted: String,
    pub modified_str: String,
    pub permissions: String,
    pub icon: String,
}

impl FileEntry {
    pub fn new(
        name: impl Into<String>,
        path: impl Into<String>,
        is_dir: bool,
        size_bytes: u64,
        modified_str: impl Into<String>,
        permissions: impl Into<String>,
    ) -> Self {
        let name_str = name.into();
        let icon = detect_file_icon(is_dir, &name_str).to_string();
        let size_formatted = if is_dir {
            "--".to_string()
        } else {
            format_file_size(size_bytes)
        };

        Self {
            name: name_str,
            path: path.into(),
            is_dir,
            size_bytes,
            size_formatted,
            modified_str: modified_str.into(),
            permissions: permissions.into(),
            icon,
        }
    }
}

pub fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn detect_file_icon(is_dir: bool, filename: &str) -> &'static str {
    if is_dir {
        "📁"
    } else {
        let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();
        match ext.as_str() {
            "rs" | "go" | "py" | "c" | "cpp" | "js" | "ts" | "html" | "css" | "json" | "yaml"
            | "yml" | "toml" | "sh" | "bash" | "conf" | "cfg" => "⚙️",
            "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" => "🖼️",
            "tar" | "gz" | "zip" | "bz2" | "xz" | "7z" | "zst" => "📦",
            "log" | "txt" | "md" | "doc" | "pdf" => "📝",
            "pem" | "pub" | "key" | "crt" | "cer" => "🔑",
            "iso" | "img" | "qcow2" | "vmdk" => "💾",
            _ => "📄",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferStatus {
    Queued,
    Transferring,
    Completed,
    Failed,
    Paused,
}

impl TransferStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Transferring => "Transferring",
            Self::Completed => "Completed",
            Self::Failed => "Failed",
            Self::Paused => "Paused",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferTask {
    pub id: String,
    pub name: String,
    pub source: String,
    pub destination: String,
    pub size_bytes: u64,
    pub transferred_bytes: u64,
    pub progress: f32, // 0.0 to 1.0
    pub speed_str: String,
    pub is_upload: bool,
    pub status: TransferStatus,
}
