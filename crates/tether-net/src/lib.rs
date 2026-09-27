//! Tether Network Protocol abstractions (SSH, Telnet, FTP, SFTP, VNC, RDP)

pub mod protocol;
pub mod session;
pub mod ssh;
pub mod terminal;

pub use protocol::{get_descriptor, ProtocolDescriptor};
pub use session::{MockInteractiveShell, SessionError, TerminalSession};
pub use terminal::TerminalScreen;
