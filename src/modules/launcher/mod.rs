use gpui::*;
use std::process::Command;

actions!(launcher, [Quit, Backspace, Up, Down, Launch]);

// Nord Dark palette
const NORD0: u32 = 0x2e3440;
const NORD1: u32 = 0x3b4252;
const NORD4: u32 = 0xd8dee9;
const NORD9: u32 = 0x81a1c1;
const NORD10: u32 = 0x5e81ac;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ApplicationInfo {
    pub name: String,
    pub name_lower: String,
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<String>,
}

pub struct Launcher {
    pub focus_handle: FocusHandle,
    applications: Vec<ApplicationInfo>,
    query: SharedString,
    query_lower: String,
    selected_index: usize,
    filtered_cache: Option<Vec<usize>>,
    last_query: String,
}

impl Launcher {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let applications = load_applications();
        Self {
            focus_handle: cx.focus_handle(),
            applications,
            query: "".into(),
            query_lower: String::new(),
            selected_index: 0,
            filtered_cache: None,
            last_query: String::new(),
        }
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        let mut query = self.query.to_string();
        if !query.is_empty() {
            query.pop();
            self.query = query.clone().into();
            self.query_lower = query.to_lowercase();
            self.selected_index = 0;
            cx.notify();
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        let filtered_count = self.filtered_apps().len();
        if filtered_count > 0 && self.selected_index > 0 {
            self.selected_index -= 1;
            cx.notify();
        }
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        let filtered_count = self.filtered_apps().len();
        if filtered_count > 0 && self.selected_index + 1 < filtered_count {
            self.selected_index += 1;
            cx.notify();
        }
    }

    fn launch(&mut self, _: &Launch, _: &mut Window, cx: &mut Context<Self>) {
        let selected_index = self.selected_index;
        let filtered_apps = self.filtered_apps();
        if let Some(app) = filtered_apps.get(selected_index) {
            let exec = app.exec.clone();
            
            std::thread::spawn(move || {
                let mut cmd = Command::new("sh");
                cmd.arg("-c")
                   .arg(&exec)
                   .env_clear()
                   .env("PATH", std::env::var("PATH").unwrap_or_default())
                   .env("HOME", std::env::var("HOME").unwrap_or_default())
                   .env("USER", std::env::var("USER").unwrap_or_default())
                   .env("XDG_RUNTIME_DIR", std::env::var("XDG_RUNTIME_DIR").unwrap_or_default())
                   .env("WAYLAND_DISPLAY", std::env::var("WAYLAND_DISPLAY").unwrap_or_default())
                   .env("DISPLAY", std::env::var("DISPLAY").unwrap_or_default())
                   .stdin(std::process::Stdio::null())
                   .stdout(std::process::Stdio::null())
                   .stderr(std::process::Stdio::null());
                
                if let Err(err) = cmd.spawn() {
                    eprintln!("Failed to launch {}: {}", exec, err);
                }
            });
            
            cx.quit();
        }
    }

    fn quit(&mut self, _: &Quit, _: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn filtered_apps(&mut self) -> Vec<&ApplicationInfo> {
        if self.query.is_empty() {
            self.filtered_cache = None;
            self.last_query.clear();
            return self.applications.iter().collect();
        }
        
        if self.last_query != self.query_lower {
            self.last_query = self.query_lower.clone();
            
            let indices: Vec<usize> = self.applications
                .iter()
                .enumerate()
                .filter_map(|(i, app)| {
                    if app.name_lower.contains(&self.query_lower) {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect();
            self.filtered_cache = Some(indices);
        }
        
        if let Some(ref indices) = self.filtered_cache {
            indices.iter().map(|&i| &self.applications[i]).collect()
        } else {
            Vec::new()
        }
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_index = self.selected_index;
        let query_text = self.query.to_string();
        let focus_handle = self.focus_handle.clone();
        
        let filtered_apps = self.filtered_apps();
        let start_index = if selected_index >= 10 { selected_index - 9 } else { 0 };

        div()
            .track_focus(&focus_handle)
            .size_full()
            .bg(rgb(NORD0))
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if let Some(key_char) = event.keystroke.key.chars().next() {
                    if key_char.is_alphanumeric() || key_char == ' ' {
                        let mut query = this.query.to_string();
                        query.push(key_char);
                        this.query = query.clone().into();
                        this.query_lower = query.to_lowercase();
                        this.selected_index = 0;
                        cx.notify();
                    }
                }
            }))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::launch))
            .on_action(cx.listener(Self::quit))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(
                        div()
                            .p_2()
                            .bg(rgb(NORD1))
                            .rounded_md()
                            .text_color(rgb(NORD4))
                            .child(format!("🔍 {}", query_text))
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .mt_2()
                            .children(
                                filtered_apps
                                    .into_iter()
                                    .enumerate()
                                    .skip(start_index)
                                    .take(10)
                                    .map(|(i, app)| {
                                        let mut item = div()
                                            .flex()
                                            .items_center()
                                            .p_2()
                                            .text_color(rgb(NORD4));
                                        
                                        if i == selected_index {
                                            item = item.bg(rgb(NORD10));
                                        }
                                        
                                        item.child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .gap_2()
                                                .child(
                                                    if let Some(icon_path) = &app.icon_path {
                                                        div()
                                                            .size_6()
                                                            .child(img(std::path::PathBuf::from(icon_path)).size_6())
                                                    } else {
                                                        div()
                                                            .size_6()
                                                            .bg(rgb(NORD9))
                                                            .rounded_sm()
                                                    }
                                                )
                                                .child(app.name.clone())
                                        )
                                    })
                            )
                    )
            )
    }
}

fn load_applications() -> Vec<ApplicationInfo> {
    use freedesktop_desktop_entry::{DesktopEntry, Iter};
    use freedesktop_icons::lookup;
    use std::fs;
    use std::collections::HashSet;
    
    let mut applications = Vec::new();
    let mut seen_names = HashSet::new();
    
    for path in Iter::new(freedesktop_desktop_entry::default_paths()) {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(desktop_entry) = DesktopEntry::decode(&path, &content) {
                if let Some(name) = desktop_entry.name(None) {
                    if let Some(exec) = desktop_entry.exec() {
                        if seen_names.contains(&name.to_string()) {
                            continue;
                        }
                        seen_names.insert(name.to_string());
                        
                        let icon_path = desktop_entry.icon()
                            .and_then(|icon_name| lookup(icon_name).with_size(24).find())
                            .map(|p| p.to_string_lossy().to_string());
                        
                        applications.push(ApplicationInfo {
                            name: name.to_string(),
                            name_lower: name.to_lowercase(),
                            exec: exec.to_string(),
                            icon: desktop_entry.icon().map(|s| s.to_string()),
                            icon_path,
                        });
                    }
                }
            }
        }
    }
    
    applications.sort_by(|a, b| a.name.cmp(&b.name));
    applications
}
