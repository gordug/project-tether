// Prevent console window in addition to Slint window in Windows release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use slint::{Color, ComponentHandle, ModelRc, SharedString, VecModel};
use tether_core::models::{AuthMethod, HostRecord, ProtocolType};
use tether_core::storage::ConfigManager;
use tether_net::session::{MockInteractiveShell, TerminalSession};
use tether_net::terminal::TerminalScreen;

slint::include_modules!();

fn proto_color(proto: ProtocolType) -> Color {
    match proto {
        ProtocolType::Ssh => Color::from_rgb_u8(56, 139, 253),     // Blue
        ProtocolType::Telnet => Color::from_rgb_u8(210, 153, 34),  // Amber
        ProtocolType::Sftp => Color::from_rgb_u8(46, 160, 67),     // Green
        ProtocolType::Scp => Color::from_rgb_u8(26, 188, 156),     // Teal
        ProtocolType::Ftp => Color::from_rgb_u8(240, 136, 62),     // Orange
        ProtocolType::Vnc => Color::from_rgb_u8(57, 197, 187),     // Cyan
        ProtocolType::Rdp => Color::from_rgb_u8(163, 113, 247),    // Purple
    }
}

fn convert_host_record(rec: &HostRecord) -> HostItem {
    let badge_color = proto_color(rec.protocol);
    let status_color = Color::from_rgb_u8(16, 185, 129); // Green online

    HostItem {
        id: SharedString::from(&rec.id),
        name: SharedString::from(&rec.name),
        group_name: SharedString::from(&rec.group),
        protocol: SharedString::from(rec.protocol.as_str()),
        host: SharedString::from(&rec.host),
        port: rec.port as i32,
        username: SharedString::from(&rec.username),
        tags: SharedString::from(rec.tags.join(", ")),
        notes: SharedString::from(rec.notes.clone().unwrap_or_default()),
        last_connected: SharedString::from(rec.last_connected.clone().unwrap_or_else(|| "Never".into())),
        is_favorite: rec.is_favorite,
        badge_color,
        status_text: SharedString::from("Ready"),
        status_color,
    }
}

fn filter_records(
    records: &[HostRecord],
    query: &str,
    category: i32,
) -> Vec<HostItem> {
    let q = query.trim().to_lowercase();
    records
        .iter()
        .filter(|r| {
            // Category filter: 0=All, 1=Term, 2=Files, 3=Desktop
            let match_cat = match category {
                1 => matches!(r.protocol, ProtocolType::Ssh | ProtocolType::Telnet),
                2 => matches!(r.protocol, ProtocolType::Sftp | ProtocolType::Scp | ProtocolType::Ftp),
                3 => matches!(r.protocol, ProtocolType::Vnc | ProtocolType::Rdp),
                _ => true,
            };
            if !match_cat {
                return false;
            }

            if q.is_empty() {
                return true;
            }

            r.name.to_lowercase().contains(&q)
                || r.host.to_lowercase().contains(&q)
                || r.username.to_lowercase().contains(&q)
                || r.protocol.as_str().to_lowercase().contains(&q)
                || r.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .map(convert_host_record)
        .collect()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    let config_mgr = Arc::new(ConfigManager::new()?);
    let all_hosts = Arc::new(Mutex::new(config_mgr.load_hosts()?));
    let open_tabs: Arc<Mutex<Vec<TabItem>>> = Arc::new(Mutex::new(Vec::new()));

    let current_query = Arc::new(Mutex::new(String::new()));
    let current_filter = Arc::new(Mutex::new(0i32));

    // Terminal session & VT100 screen buffer
    let terminal_screen = Arc::new(Mutex::new(TerminalScreen::new(24, 90)));
    let active_terminal_session: Arc<Mutex<Option<Arc<dyn TerminalSession>>>> = Arc::new(Mutex::new(None));

    // Populate initial hosts in sidebar
    {
        let hosts_guard = all_hosts.lock().unwrap();
        let items = filter_records(&hosts_guard, "", 0);
        let model = Rc::new(VecModel::from(items));
        ui.global::<AppState>().set_hosts(ModelRc::from(model));
    }

    // Set initial metrics
    ui.global::<AppState>().set_metrics(ServerMetrics {
        cpu_usage: 0.28,
        mem_usage: 0.54,
        disk_usage: 0.62,
        ping_ms: 14,
        uptime_str: SharedString::from("42d 03h 14m"),
        load_avg: SharedString::from("0.24, 0.18, 0.12"),
    });

    // Helper to refresh host model
    let refresh_hosts_ui = {
        let ui_handle = ui.as_weak();
        let all_hosts = Arc::clone(&all_hosts);
        let current_query = Arc::clone(&current_query);
        let current_filter = Arc::clone(&current_filter);

        move || {
            if let Some(ui) = ui_handle.upgrade() {
                let hosts_guard = all_hosts.lock().unwrap();
                let q = current_query.lock().unwrap().clone();
                let f = *current_filter.lock().unwrap();
                let items = filter_records(&hosts_guard, &q, f);
                ui.global::<AppState>().set_hosts(ModelRc::new(VecModel::from(items)));
            }
        }
    };

    // Helper to start an interactive terminal session for a tab
    let start_terminal_for_tab = {
        let ui_handle = ui.as_weak();
        let terminal_screen = Arc::clone(&terminal_screen);
        let active_terminal_session = Arc::clone(&active_terminal_session);

        move |host_name: String, user_name: String| {
            // Reset screen
            {
                let mut screen = terminal_screen.lock().unwrap();
                *screen = TerminalScreen::new(24, 90);
            }

            let ui_for_output = ui_handle.clone();
            let screen_for_output = Arc::clone(&terminal_screen);

            let session = MockInteractiveShell::spawn(
                host_name.clone(),
                user_name.clone(),
                move |bytes: Vec<u8>| {
                    let mut screen = screen_for_output.lock().unwrap();
                    screen.process(&bytes);
                    let rendered = screen.render_rows();

                    let ui_weak = ui_for_output.clone();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_weak.upgrade() {
                            let slint_lines: Vec<SharedString> =
                                rendered.into_iter().map(SharedString::from).collect();
                            ui.global::<AppState>().set_terminal_lines(ModelRc::new(VecModel::from(slint_lines)));
                        }
                    });
                },
            );

            *active_terminal_session.lock().unwrap() = Some(Arc::new(session));

            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_terminal_prompt(SharedString::from(format!("{}@{}:~$ ", user_name, host_name)));
            }
        }
    };

    // Helper to refresh tabs UI
    let refresh_tabs_ui = {
        let ui_handle = ui.as_weak();
        let open_tabs = Arc::clone(&open_tabs);
        let start_term = start_terminal_for_tab.clone();

        move |active_idx: i32| {
            if let Some(ui) = ui_handle.upgrade() {
                let mut tabs_guard = open_tabs.lock().unwrap();
                for (i, tab) in tabs_guard.iter_mut().enumerate() {
                    tab.is_active = (i as i32) == active_idx;
                }

                let has_active = !tabs_guard.is_empty() && active_idx >= 0 && (active_idx as usize) < tabs_guard.len();
                if has_active {
                    let active_item = tabs_guard[active_idx as usize].clone();
                    ui.global::<AppState>().set_active_tab(active_item.clone());
                    start_term(active_item.title.to_string(), "admin".to_string());
                }

                ui.global::<AppState>().set_has_active_tab(has_active);
                ui.global::<AppState>().set_active_tab_index(active_idx);
                ui.global::<AppState>().set_tabs(ModelRc::new(VecModel::from(tabs_guard.clone())));
            }
        }
    };

    // Callback: Host Selected in Sidebar
    {
        let ui_handle = ui.as_weak();
        ui.global::<AppLogic>().on_host_selected(move |idx| {
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_selected_host_index(idx);
            }
        });
    }

    // Callback: Connect clicked on host card
    {
        let all_hosts = Arc::clone(&all_hosts);
        let current_query = Arc::clone(&current_query);
        let current_filter = Arc::clone(&current_filter);
        let open_tabs = Arc::clone(&open_tabs);
        let refresh_tabs = refresh_tabs_ui.clone();

        ui.global::<AppLogic>().on_host_connect(move |idx| {
            let hosts_guard = all_hosts.lock().unwrap();
            let q = current_query.lock().unwrap().clone();
            let f = *current_filter.lock().unwrap();
            let filtered = filter_records(&hosts_guard, &q, f);

            if let Some(selected) = filtered.get(idx as usize) {
                let mut tabs = open_tabs.lock().unwrap();
                // Check if already open
                if let Some(existing_idx) = tabs.iter().position(|t| t.host_id == selected.id) {
                    drop(tabs);
                    refresh_tabs(existing_idx as i32);
                } else {
                    let new_tab = TabItem {
                        id: SharedString::from(uuid::Uuid::new_v4().to_string()),
                        host_id: selected.id.clone(),
                        title: selected.name.clone(),
                        protocol: selected.protocol.clone(),
                        badge_color: selected.badge_color,
                        is_active: true,
                    };
                    tabs.push(new_tab);
                    let new_idx = (tabs.len() - 1) as i32;
                    drop(tabs);
                    refresh_tabs(new_idx);
                }
            }
        });
    }

    // Callback: Tab Selected
    {
        let refresh_tabs = refresh_tabs_ui.clone();
        ui.global::<AppLogic>().on_tab_selected(move |idx| {
            refresh_tabs(idx);
        });
    }

    // Callback: Tab Closed
    {
        let open_tabs = Arc::clone(&open_tabs);
        let refresh_tabs = refresh_tabs_ui.clone();
        ui.global::<AppLogic>().on_tab_closed(move |idx| {
            let mut tabs = open_tabs.lock().unwrap();
            if idx >= 0 && (idx as usize) < tabs.len() {
                tabs.remove(idx as usize);
                let new_active = if tabs.is_empty() {
                    -1
                } else if idx >= (tabs.len() as i32) {
                    (tabs.len() - 1) as i32
                } else {
                    idx
                };
                drop(tabs);
                refresh_tabs(new_active);
            }
        });
    }

    // Callback: Search Query Changed
    {
        let current_query = Arc::clone(&current_query);
        let refresh_hosts = refresh_hosts_ui.clone();
        let ui_handle = ui.as_weak();
        ui.global::<AppLogic>().on_search_changed(move |text| {
            *current_query.lock().unwrap() = text.to_string();
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_search_text(text);
            }
            refresh_hosts();
        });
    }

    // Callback: Filter Category Changed
    {
        let current_filter = Arc::clone(&current_filter);
        let refresh_hosts = refresh_hosts_ui.clone();
        let ui_handle = ui.as_weak();
        ui.global::<AppLogic>().on_filter_changed(move |cat| {
            *current_filter.lock().unwrap() = cat;
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_category_filter(cat);
            }
            refresh_hosts();
        });
    }

    // Callback: Quick Connect from Search/Run Bar
    {
        let open_tabs = Arc::clone(&open_tabs);
        let refresh_tabs = refresh_tabs_ui.clone();
        ui.global::<AppLogic>().on_quick_connect_submit(move |query| {
            let q = query.trim();
            if q.is_empty() {
                return;
            }

            let (proto, target) = if let Some(stripped) = q.strip_prefix("ssh://") {
                ("SSH", stripped)
            } else if let Some(stripped) = q.strip_prefix("vnc://") {
                ("VNC", stripped)
            } else if let Some(stripped) = q.strip_prefix("rdp://") {
                ("RDP", stripped)
            } else if let Some(stripped) = q.strip_prefix("telnet://") {
                ("Telnet", stripped)
            } else if let Some(stripped) = q.strip_prefix("sftp://") {
                ("SFTP", stripped)
            } else {
                ("SSH", q)
            };

            let tab = TabItem {
                id: SharedString::from(uuid::Uuid::new_v4().to_string()),
                host_id: SharedString::from(format!("quick-{}", target)),
                title: SharedString::from(format!("Quick: {}", target)),
                protocol: SharedString::from(proto),
                badge_color: proto_color(ProtocolType::from_str_lenient(proto)),
                is_active: true,
            };

            let mut tabs = open_tabs.lock().unwrap();
            tabs.push(tab);
            let idx = (tabs.len() - 1) as i32;
            drop(tabs);
            refresh_tabs(idx);
        });
    }

    // Callback: Quick Connect Protocol from Hero Card
    {
        let open_tabs = Arc::clone(&open_tabs);
        let refresh_tabs = refresh_tabs_ui.clone();
        ui.global::<AppLogic>().on_quick_connect_protocol(move |proto| {
            let proto_type = ProtocolType::from_str_lenient(&proto);
            let tab = TabItem {
                id: SharedString::from(uuid::Uuid::new_v4().to_string()),
                host_id: SharedString::from(format!("quick-{}", proto)),
                title: SharedString::from(format!("New {} Session", proto)),
                protocol: proto,
                badge_color: proto_color(proto_type),
                is_active: true,
            };

            let mut tabs = open_tabs.lock().unwrap();
            tabs.push(tab);
            let idx = (tabs.len() - 1) as i32;
            drop(tabs);
            refresh_tabs(idx);
        });
    }

    // Callback: Open & Cancel New Host Modal
    {
        let ui_handle = ui.as_weak();
        ui.global::<AppLogic>().on_open_new_host_modal(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_show_new_host_dialog(true);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        ui.global::<AppLogic>().on_cancel_new_host_modal(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_show_new_host_dialog(false);
            }
        });
    }

    // Callback: Save New Host Connection
    {
        let ui_handle = ui.as_weak();
        let all_hosts = Arc::clone(&all_hosts);
        let config_mgr = Arc::clone(&config_mgr);
        let refresh_hosts = refresh_hosts_ui.clone();

        ui.global::<AppLogic>().on_save_new_host(
            move |name, proto_str, host, port_str, username, group, tags_str| {
                let proto = ProtocolType::from_str_lenient(&proto_str);
                let port = port_str.trim().parse::<u16>().unwrap_or_else(|_| proto.default_port());

                let tags: Vec<String> = tags_str
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let new_record = HostRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: if name.trim().is_empty() { host.to_string() } else { name.to_string() },
                    group: if group.trim().is_empty() { "Default".into() } else { group.to_string() },
                    protocol: proto,
                    host: host.to_string(),
                    port,
                    username: username.to_string(),
                    auth_method: AuthMethod::Password,
                    key_path: None,
                    tags,
                    notes: None,
                    last_connected: None,
                    is_favorite: false,
                };

                {
                    let mut hosts_guard = all_hosts.lock().unwrap();
                    hosts_guard.push(new_record);
                    let _ = config_mgr.save_hosts(&hosts_guard);
                }

                refresh_hosts();

                if let Some(ui) = ui_handle.upgrade() {
                    ui.global::<AppState>().set_show_new_host_dialog(false);
                }
            },
        );
    }

    // Callback: Disconnect
    {
        let open_tabs = Arc::clone(&open_tabs);
        let refresh_tabs = refresh_tabs_ui.clone();
        let ui_handle = ui.as_weak();
        ui.global::<AppLogic>().on_disconnect_session(move || {
            if let Some(ui) = ui_handle.upgrade() {
                let active_idx = ui.global::<AppState>().get_active_tab_index();
                let mut tabs = open_tabs.lock().unwrap();
                if active_idx >= 0 && (active_idx as usize) < tabs.len() {
                    tabs.remove(active_idx as usize);
                    let new_active = if tabs.is_empty() {
                        -1
                    } else if active_idx >= (tabs.len() as i32) {
                        (tabs.len() - 1) as i32
                    } else {
                        active_idx
                    };
                    drop(tabs);
                    refresh_tabs(new_active);
                }
            }
        });
    }

    // Callback: Refresh session / Telemetry
    {
        let ui_handle = ui.as_weak();
        ui.global::<AppLogic>().on_refresh_session(move || {
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_metrics(ServerMetrics {
                    cpu_usage: 0.35,
                    mem_usage: 0.58,
                    disk_usage: 0.62,
                    ping_ms: 12,
                    uptime_str: SharedString::from("42d 03h 15m"),
                    load_avg: SharedString::from("0.31, 0.22, 0.15"),
                });
            }
        });
    }

    // Terminal Callbacks
    {
        let active_session = Arc::clone(&active_terminal_session);
        ui.global::<AppLogic>().on_terminal_input_submitted(move |cmd| {
            let session_guard = active_session.lock().unwrap();
            if let Some(ref session) = *session_guard {
                let session = Arc::clone(session);
                let payload = format!("{}\r\n", cmd).into_bytes();
                tokio::spawn(async move {
                    let _ = session.send_input(&payload).await;
                });
            }
        });
    }

    {
        let active_session = Arc::clone(&active_terminal_session);
        ui.global::<AppLogic>().on_terminal_quick_cmd(move |cmd| {
            let session_guard = active_session.lock().unwrap();
            if let Some(ref session) = *session_guard {
                let session = Arc::clone(session);
                let payload = format!("{}\r\n", cmd).into_bytes();
                tokio::spawn(async move {
                    let _ = session.send_input(&payload).await;
                });
            }
        });
    }

    {
        let active_session = Arc::clone(&active_terminal_session);
        let terminal_screen = Arc::clone(&terminal_screen);
        let ui_handle = ui.as_weak();

        ui.global::<AppLogic>().on_terminal_clear_requested(move || {
            {
                let mut screen = terminal_screen.lock().unwrap();
                screen.process(b"\x1b[2J\x1b[H");
            }
            let session_guard = active_session.lock().unwrap();
            if let Some(ref session) = *session_guard {
                let session = Arc::clone(session);
                tokio::spawn(async move {
                    let _ = session.send_input(b"clear\r\n").await;
                });
            }
            if let Some(ui) = ui_handle.upgrade() {
                ui.global::<AppState>().set_terminal_lines(ModelRc::new(VecModel::from(Vec::new())));
            }
        });
    }

    // Live Telemetry Jitter Timer (updates metrics every 3 seconds to show live reactive dials)
    let _telemetry_timer = {
        let ui_handle = ui.as_weak();
        let mut tick = 0u64;
        let timer = slint::Timer::default();
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_millis(3000),
            move || {
                tick += 1;
                if let Some(ui) = ui_handle.upgrade() {
                    let cpu_variation = 0.20 + ((tick % 7) as f32) * 0.08;
                    let mem_variation = 0.52 + ((tick % 4) as f32) * 0.02;
                    let ping = 11 + ((tick % 5) as i32) * 2;
                    ui.global::<AppState>().set_metrics(ServerMetrics {
                        cpu_usage: cpu_variation.min(0.95),
                        mem_usage: mem_variation.min(0.90),
                        disk_usage: 0.62,
                        ping_ms: ping,
                        uptime_str: SharedString::from("42d 03h 16m"),
                        load_avg: SharedString::from(format!("{:.2}, 0.20, 0.15", cpu_variation)),
                    });
                }
            },
        );
        timer
    };

    println!("⚡ Tether Application Starting...");
    ui.run()?;

    Ok(())
}
