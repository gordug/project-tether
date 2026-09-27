use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tether_core::models::FileEntry;

pub struct LocalFileSystem;

impl LocalFileSystem {
    /// List entries in a local directory
    pub fn list_dir(dir: &Path) -> Result<Vec<FileEntry>, std::io::Error> {
        let read_dir = fs::read_dir(dir)?;
        let mut entries = Vec::new();

        for item in read_dir {
            if let Ok(entry) = item {
                let name = entry.file_name().to_string_lossy().to_string();
                let path = entry.path().to_string_lossy().to_string();
                let metadata = entry.metadata().ok();

                let is_dir = metadata.as_ref().map(|m| m.is_dir()).unwrap_or(false);
                let size_bytes = metadata.as_ref().map(|m| m.len()).unwrap_or(0);

                let modified_str = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .map(format_system_time)
                    .unwrap_or_else(|| "Unknown".to_string());

                let permissions = if is_dir {
                    "drwxr-xr-x".to_string()
                } else {
                    "-rw-r--r--".to_string()
                };

                entries.push(FileEntry::new(
                    name,
                    path,
                    is_dir,
                    size_bytes,
                    modified_str,
                    permissions,
                ));
            }
        }

        // Sort: directories first (alphabetical), then files (alphabetical)
        entries.sort_by(|a, b| {
            if a.is_dir == b.is_dir {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            } else if a.is_dir {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Greater
            }
        });

        Ok(entries)
    }

    /// Get parent directory path or current root
    pub fn parent_dir(path: &Path) -> PathBuf {
        path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| path.to_path_buf())
    }
}

fn format_system_time(st: std::time::SystemTime) -> String {
    if let Ok(duration) = st.duration_since(std::time::UNIX_EPOCH) {
        let secs = duration.as_secs();
        let days = secs / 86400;
        let rem_secs = secs % 86400;
        let hours = rem_secs / 3600;
        let mins = (rem_secs % 3600) / 60;
        let approx_year = 1970 + (days * 400 / 146097);
        format!("{}-{:02} {:02}:{:02}", approx_year, (days % 365) / 30 + 1, hours, mins)
    } else {
        "Recent".to_string()
    }
}

/// Simulated remote directory and file system for SFTP, SCP, and FTP sessions
#[derive(Debug, Clone)]
pub struct MockRemoteFileSystem {
    directories: HashMap<String, Vec<FileEntry>>,
}

impl Default for MockRemoteFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MockRemoteFileSystem {
    pub fn new() -> Self {
        let mut dirs = HashMap::new();

        // Root /
        dirs.insert(
            "/".to_string(),
            vec![
                FileEntry::new("bin", "/bin", true, 4096, "2026-09-01 08:00", "drwxr-xr-x"),
                FileEntry::new("etc", "/etc", true, 4096, "2026-09-15 11:20", "drwxr-xr-x"),
                FileEntry::new("home", "/home", true, 4096, "2026-09-20 14:10", "drwxr-xr-x"),
                FileEntry::new("mnt", "/mnt", true, 4096, "2026-09-10 09:30", "drwxr-xr-x"),
                FileEntry::new("srv", "/srv", true, 4096, "2026-09-12 16:45", "drwxr-xr-x"),
                FileEntry::new("var", "/var", true, 4096, "2026-09-27 10:15", "drwxr-xr-x"),
                FileEntry::new("tmp", "/tmp", true, 4096, "2026-09-27 17:00", "drwxrwxrwt"),
            ],
        );

        // /home
        dirs.insert(
            "/home".to_string(),
            vec![
                FileEntry::new("admin", "/home/admin", true, 4096, "2026-09-20 14:10", "drwxr-xr-x"),
                FileEntry::new("backup", "/home/backup", true, 4096, "2026-09-18 10:00", "drwxr-xr-x"),
            ],
        );

        // /home/admin
        dirs.insert(
            "/home/admin".to_string(),
            vec![
                FileEntry::new(".ssh", "/home/admin/.ssh", true, 4096, "2026-09-20 14:15", "drwx------"),
                FileEntry::new("projects", "/home/admin/projects", true, 4096, "2026-09-25 18:30", "drwxr-xr-x"),
                FileEntry::new("backups", "/home/admin/backups", true, 4096, "2026-09-26 04:00", "drwxr-xr-x"),
                FileEntry::new("docker-compose.yml", "/home/admin/docker-compose.yml", false, 2450, "2026-09-24 12:40", "-rw-r--r--"),
                FileEntry::new("deploy-cluster.sh", "/home/admin/deploy-cluster.sh", false, 6820, "2026-09-22 15:10", "-rwxr-xr-x"),
                FileEntry::new("server-config.toml", "/home/admin/server-config.toml", false, 1890, "2026-09-26 09:12", "-rw-r--r--"),
                FileEntry::new("application.log", "/home/admin/application.log", false, 4820190, "2026-09-27 16:50", "-rw-r--r--"),
                FileEntry::new("database-backup.sql.gz", "/home/admin/database-backup.sql.gz", false, 24891000, "2026-09-26 02:00", "-rw-------"),
            ],
        );

        // /etc
        dirs.insert(
            "/etc".to_string(),
            vec![
                FileEntry::new("nginx", "/etc/nginx", true, 4096, "2026-09-15 11:20", "drwxr-xr-x"),
                FileEntry::new("ssh", "/etc/ssh", true, 4096, "2026-09-01 08:00", "drwxr-xr-x"),
                FileEntry::new("hosts", "/etc/hosts", false, 412, "2026-09-01 08:00", "-rw-r--r--"),
                FileEntry::new("resolv.conf", "/etc/resolv.conf", false, 182, "2026-09-27 00:00", "-rw-r--r--"),
            ],
        );

        // /etc/nginx
        dirs.insert(
            "/etc/nginx".to_string(),
            vec![
                FileEntry::new("conf.d", "/etc/nginx/conf.d", true, 4096, "2026-09-15 11:20", "drwxr-xr-x"),
                FileEntry::new("sites-available", "/etc/nginx/sites-available", true, 4096, "2026-09-15 11:20", "drwxr-xr-x"),
                FileEntry::new("sites-enabled", "/etc/nginx/sites-enabled", true, 4096, "2026-09-15 11:20", "drwxr-xr-x"),
                FileEntry::new("nginx.conf", "/etc/nginx/nginx.conf", false, 3240, "2026-09-15 11:20", "-rw-r--r--"),
                FileEntry::new("mime.types", "/etc/nginx/mime.types", false, 5230, "2026-09-15 11:20", "-rw-r--r--"),
            ],
        );

        // /var/log
        dirs.insert(
            "/var/log".to_string(),
            vec![
                FileEntry::new("nginx", "/var/log/nginx", true, 4096, "2026-09-27 10:15", "drwxr-xr-x"),
                FileEntry::new("syslog", "/var/log/syslog", false, 15420100, "2026-09-27 17:15", "-rw-r-----"),
                FileEntry::new("auth.log", "/var/log/auth.log", false, 3210400, "2026-09-27 16:30", "-rw-r-----"),
                FileEntry::new("tether-daemon.log", "/var/log/tether-daemon.log", false, 829100, "2026-09-27 17:10", "-rw-r--r--"),
            ],
        );

        // /mnt/storage
        dirs.insert(
            "/mnt/storage".to_string(),
            vec![
                FileEntry::new("vm-disk-images", "/mnt/storage/vm-disk-images", true, 4096, "2026-09-10 09:30", "drwxr-xr-x"),
                FileEntry::new("nightly-snapshots", "/mnt/storage/nightly-snapshots", true, 4096, "2026-09-27 03:00", "drwxr-xr-x"),
                FileEntry::new("archival_dataset.tar.zst", "/mnt/storage/archival_dataset.tar.zst", false, 1849204000, "2026-09-25 14:00", "-rw-r--r--"),
                FileEntry::new("tether-cluster-backup.qcow2", "/mnt/storage/tether-cluster-backup.qcow2", false, 8492040000, "2026-09-26 12:00", "-rw-r--r--"),
            ],
        );

        Self { directories: dirs }
    }

    pub fn list_dir(&self, path: &str) -> Vec<FileEntry> {
        let norm = Self::normalize_path(path);
        if let Some(entries) = self.directories.get(&norm) {
            let mut result = entries.clone();
            result.sort_by(|a, b| {
                if a.is_dir == b.is_dir {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                } else if a.is_dir {
                    std::cmp::Ordering::Less
                } else {
                    std::cmp::Ordering::Greater
                }
            });
            result
        } else {
            Vec::new()
        }
    }

    pub fn parent_dir(path: &str) -> String {
        let norm = Self::normalize_path(path);
        if norm == "/" {
            return "/".to_string();
        }
        let parts: Vec<&str> = norm.trim_end_matches('/').split('/').collect();
        if parts.len() <= 2 {
            "/".to_string()
        } else {
            parts[..parts.len() - 1].join("/")
        }
    }

    pub fn normalize_path(path: &str) -> String {
        let p = path.trim();
        if p.is_empty() || p == "/" {
            "/".to_string()
        } else if !p.starts_with('/') {
            format!("/{}", p.trim_end_matches('/'))
        } else {
            p.trim_end_matches('/').to_string()
        }
    }

    pub fn add_file(&mut self, dir_path: &str, file: FileEntry) {
        let norm = Self::normalize_path(dir_path);
        self.directories.entry(norm).or_default().push(file);
    }

    pub fn make_dir(&mut self, dir_path: &str, name: &str) {
        let norm = Self::normalize_path(dir_path);
        let new_path = if norm == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", norm, name)
        };

        let new_entry = FileEntry::new(name, &new_path, true, 4096, "Just now", "drwxr-xr-x");
        self.directories.entry(norm).or_default().push(new_entry);
        self.directories.entry(new_path).or_default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remote_file_system_nav() {
        let mut fs = MockRemoteFileSystem::new();
        let home_entries = fs.list_dir("/home/admin");
        assert!(!home_entries.is_empty());
        assert!(home_entries.iter().any(|e| e.name == "docker-compose.yml"));

        let parent = MockRemoteFileSystem::parent_dir("/home/admin");
        assert_eq!(parent, "/home");

        let root_parent = MockRemoteFileSystem::parent_dir("/");
        assert_eq!(root_parent, "/");

        fs.make_dir("/home/admin", "test_new_folder");
        let updated = fs.list_dir("/home/admin");
        assert!(updated.iter().any(|e| e.name == "test_new_folder" && e.is_dir));
    }

    #[test]
    fn test_local_file_system_listing() {
        let curr_dir = std::env::current_dir().expect("valid cwd");
        let entries = LocalFileSystem::list_dir(&curr_dir).expect("lists cwd");
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|e| e.name == "Cargo.toml"));
    }
}
