use tether_core::models::ProtocolType;

#[derive(Debug, Clone)]
pub struct ProtocolDescriptor {
    pub protocol: ProtocolType,
    pub default_port: u16,
    pub supports_pty: bool,
    pub supports_file_transfer: bool,
    pub supports_display: bool,
}

pub fn get_descriptor(proto: ProtocolType) -> ProtocolDescriptor {
    match proto {
        ProtocolType::Ssh => ProtocolDescriptor {
            protocol: proto,
            default_port: 22,
            supports_pty: true,
            supports_file_transfer: true,
            supports_display: false,
        },
        ProtocolType::Telnet => ProtocolDescriptor {
            protocol: proto,
            default_port: 23,
            supports_pty: true,
            supports_file_transfer: false,
            supports_display: false,
        },
        ProtocolType::Sftp => ProtocolDescriptor {
            protocol: proto,
            default_port: 22,
            supports_pty: false,
            supports_file_transfer: true,
            supports_display: false,
        },
        ProtocolType::Scp => ProtocolDescriptor {
            protocol: proto,
            default_port: 22,
            supports_pty: false,
            supports_file_transfer: true,
            supports_display: false,
        },
        ProtocolType::Ftp => ProtocolDescriptor {
            protocol: proto,
            default_port: 21,
            supports_pty: false,
            supports_file_transfer: true,
            supports_display: false,
        },
        ProtocolType::Vnc => ProtocolDescriptor {
            protocol: proto,
            default_port: 5900,
            supports_pty: false,
            supports_file_transfer: false,
            supports_display: true,
        },
        ProtocolType::Rdp => ProtocolDescriptor {
            protocol: proto,
            default_port: 3389,
            supports_pty: false,
            supports_file_transfer: false,
            supports_display: true,
        },
    }
}
