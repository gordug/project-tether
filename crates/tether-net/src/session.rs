use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum SessionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("SSH protocol error: {0}")]
    Ssh(String),
    #[error("Telnet protocol error: {0}")]
    Telnet(String),
    #[error("Session is closed")]
    Closed,
}

#[async_trait::async_trait]
pub trait TerminalSession: Send + Sync {
    async fn send_input(&self, data: &[u8]) -> Result<(), SessionError>;
    async fn resize(&self, rows: u16, cols: u16) -> Result<(), SessionError>;
    async fn close(&self) -> Result<(), SessionError>;
}

/// An interactive shell session that provides realistic terminal feedback,
/// command evaluation, ANSI colors, and cursor movement.
pub struct MockInteractiveShell {
    input_tx: mpsc::Sender<Vec<u8>>,
}

impl MockInteractiveShell {
    pub fn spawn<F>(host_name: String, username: String, on_output: F) -> Self
    where
        F: Fn(Vec<u8>) + Send + Sync + 'static,
    {
        let (input_tx, mut input_rx) = mpsc::channel::<Vec<u8>>(128);
        let callback = Arc::new(on_output);

        tokio::spawn(async move {
            let prompt = format!("\x1b[1;32m{}@{}:~$\x1b[0m ", username, host_name);
            let banner = format!(
                "\x1b[1;34m=== Connected to {} via Tether SSH Terminal ===\x1b[0m\r\n\
                \x1b[90mLinux tether-core 6.6.0-x86_64 #1 SMP PREEMPT_DYNAMIC GNU/Linux\x1b[0m\r\n\
                \x1b[90mSystem information as of {} (UTC)\x1b[0m\r\n\r\n\
                \x1b[36mType 'help' for available commands, 'clear' to reset screen.\x1b[0m\r\n\r\n{}",
                host_name,
                "2026-09-27 16:59:00",
                prompt
            );

            callback(banner.into_bytes());

            let mut line_buffer = String::new();

            while let Some(bytes) = input_rx.recv().await {
                for &b in &bytes {
                    match b {
                        b'\r' | b'\n' => {
                            callback(b"\r\n".to_vec());
                            let cmd = line_buffer.trim().to_string();
                            line_buffer.clear();

                            let response = Self::handle_command(&cmd, &host_name);
                            if !response.is_empty() {
                                callback(response.into_bytes());
                            }
                            callback(prompt.as_bytes().to_vec());
                        }
                        8 | 127 => {
                            // Backspace
                            if !line_buffer.is_empty() {
                                line_buffer.pop();
                                callback(b"\x08 \x08".to_vec());
                            }
                        }
                        3 => {
                            // Ctrl+C
                            line_buffer.clear();
                            callback(format!("^C\r\n{}", prompt).into_bytes());
                        }
                        b => {
                            if b.is_ascii_graphic() || b == b' ' {
                                line_buffer.push(b as char);
                                callback(vec![b]);
                            }
                        }
                    }
                }
            }
        });

        Self { input_tx }
    }

    fn handle_command(cmd: &str, _host: &str) -> String {
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return String::new();
        }

        match parts[0] {
            "help" => {
                "\x1b[1;33mAvailable Commands:\x1b[0m\r\n\
                 \x1b[32m  help     \x1b[0m - Show this list of built-in commands\r\n\
                 \x1b[32m  uname -a \x1b[0m - Print operating system kernel information\r\n\
                 \x1b[32m  uptime   \x1b[0m - Display how long the system has been running\r\n\
                 \x1b[32m  df -h    \x1b[0m - Show disk storage space usage\r\n\
                 \x1b[32m  top      \x1b[0m - Display snapshot of top running processes\r\n\
                 \x1b[32m  ping     \x1b[0m - Send simulated ICMP echo packets\r\n\
                 \x1b[32m  matrix   \x1b[0m - Play ANSI digital rain snippet\r\n\
                 \x1b[32m  clear    \x1b[0m - Clear terminal screen buffer\r\n\r\n"
                    .to_string()
            }
            "uptime" => {
                " 17:02:44 up 42 days,  3:42,  1 user,  load average: 0.14, 0.09, 0.06\r\n"
                    .to_string()
            }
            "uname" => {
                "Linux tether-srv01 6.6.0-generic #42-Ubuntu SMP Fri Sep 25 14:12:00 UTC 2026 x86_64 GNU/Linux\r\n"
                    .to_string()
            }
            "df" => {
                "\x1b[1mFilesystem      Size  Used Avail Use% Mounted on\x1b[0m\r\n\
                /dev/nvme0n1p2  468G  290G  155G  66% /\r\n\
                /dev/nvme0n1p1  511M  6.1M  505M   2% /boot/efi\r\n\
                /dev/sda1       3.6T  2.1T  1.4T  61% /mnt/storage\r\n\
                tmpfs            16G  1.2M   16G   1% /run\r\n\r\n"
                    .to_string()
            }
            "top" => {
                "\x1b[1;37mtop - 17:03:01 up 42 days, 1 user, load average: 0.14, 0.09, 0.06\x1b[0m\r\n\
                Tasks: 218 total, 1 running, 217 sleeping, 0 stopped\r\n\
                %Cpu(s):  2.3 us,  1.1 sy,  0.0 ni, 96.4 id,  0.2 wa\r\n\
                MiB Mem :  31980.4 total,  14210.1 free,  12140.2 used\r\n\r\n\
                \x1b[7m  PID USER      PR  NI    VIRT    RES    SHR S  %CPU  %MEM     TIME+ COMMAND       \x1b[0m\r\n\
                 1402 root      20   0 1824108 412080  45120 S   4.8   1.3 124:12.44 dockerd       \r\n\
                 2891 admin     20   0  891240 184512  32104 S   2.1   0.6  48:02.19 node          \r\n\
                 3104 postgres  20   0  421098 128940  89100 S   1.0   0.4  19:44.82 postgres      \r\n\
                 4119 admin     20   0   24108   4120   3100 R   0.7   0.0   0:00.12 top           \r\n\r\n"
                    .to_string()
            }
            "ping" => {
                let target = parts.get(1).copied().unwrap_or("8.8.8.8");
                format!(
                    "PING {} ({}): 56 data bytes\r\n\
                    64 bytes from {}: icmp_seq=1 ttl=118 time=14.2 ms\r\n\
                    64 bytes from {}: icmp_seq=2 ttl=118 time=13.8 ms\r\n\
                    64 bytes from {}: icmp_seq=3 ttl=118 time=14.5 ms\r\n\
                    --- {} ping statistics ---\r\n\
                    3 packets transmitted, 3 packets received, 0.0% packet loss\r\n\
                    round-trip min/avg/max = 13.8/14.1/14.5 ms\r\n\r\n",
                    target, target, target, target, target, target
                )
            }
            "matrix" => {
                "\x1b[32m01001000 01100101 01101100 01101100 01101111\r\n\
                01010100 01100101 01110100 01101000 01100101 01110010\r\n\
                [System Access Granted]\x1b[0m\r\n\r\n"
                    .to_string()
            }
            "clear" => "\x1b[2J\x1b[H".to_string(),
            unknown => {
                format!(
                    "bash: {}: command not found (type 'help' for command list)\r\n",
                    unknown
                )
            }
        }
    }
}

#[async_trait::async_trait]
impl TerminalSession for MockInteractiveShell {
    async fn send_input(&self, data: &[u8]) -> Result<(), SessionError> {
        self.input_tx
            .send(data.to_vec())
            .await
            .map_err(|_| SessionError::Closed)
    }

    async fn resize(&self, _rows: u16, _cols: u16) -> Result<(), SessionError> {
        Ok(())
    }

    async fn close(&self) -> Result<(), SessionError> {
        Ok(())
    }
}
