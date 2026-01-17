use gpui::{
    actions, background_executor, div, img, layer_shell::*, point, prelude::*, px, App,
    Application, Bounds, Context, Entity, FocusHandle, KeyBinding, KeyDownEvent, Render, SharedString,
    Size, Task, Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::services::{applications, calculator, process};
use crate::theme::Theme;
use crate::widgets::launcher::{fuzzy, state};
use applications::{load_from_cache, save_to_cache, scan_applications};
use calculator::{is_calculator_query, Calculator};
use fuzzy::FuzzyMatcher;
use process::{get_running_processes, is_process_query, kill_process, ProcessInfo};
use state::ApplicationInfo;

actions!(
    launcher,
    [
        Quit,
        Backspace,
        Up,
        Down,
        Launch,
        OpenSettings
    ]
);

enum SearchResult {
    Application(usize),
    Calculation(String),
    Process(ProcessInfo),
}

struct Launcher {
    focus_handle: FocusHandle,
    applications: Vec<ApplicationInfo>,
    query: SharedString,
    selected_index: usize,
    fuzzy_matcher: FuzzyMatcher,
    calculator: Option<Calculator>,
    search_results: Vec<SearchResult>,
    scroll_offset: usize,
    search_task: Option<Task<()>>,
    theme: Theme,
}

impl Launcher {
    fn new(cx: &mut Context<Self>) -> Self {

        let mut launcher = Self {
            focus_handle: cx.focus_handle(),
            applications: Vec::new(),
            query: "".into(),
            selected_index: 0,
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: None,
            search_results: Vec::new(),
            scroll_offset: 0,
            search_task: None,
            theme: Theme::nord_dark(),
        };

        // Try to load vault from existing session in background
        if let Some(apps) = load_from_cache() {
            launcher.applications = apps;
            launcher.fuzzy_matcher.set_candidates(&launcher.applications);
            launcher.update_search_results();
        }

        cx.spawn(async move |this, cx| {
            let apps = background_executor()
                .spawn(async move { scan_applications() })
                .await;

            this.update(cx, |this, cx| {
                this.applications = apps.clone();
                this.fuzzy_matcher.set_candidates(&this.applications);
                this.update_search_results();
                cx.notify();
            })?;

            background_executor()
                .spawn(async move {
                    let _ = save_to_cache(&apps);
                })
                .detach();

            anyhow::Ok(())
        })
        .detach();

        cx.spawn(async move |this, cx| {
            let calculator = background_executor()
                .spawn(async move { Calculator::new() })
                .await;

            this.update(cx, |this, cx| {
                this.calculator = Some(calculator);
                if is_calculator_query(&this.query) {
                    this.update_search_results();
                    cx.notify();
                }
            })?;

            anyhow::Ok(())
        })
        .detach();

        launcher
    }

    fn new_for_widget<T>(cx: &mut Context<T>) -> Self {
        let mut launcher = Self {
            focus_handle: cx.focus_handle(),
            applications: Vec::new(),
            query: "".into(),
            selected_index: 0,
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: None,
            search_results: Vec::new(),
            scroll_offset: 0,
            search_task: None,
            theme: Theme::nord_dark(),
        };

        // Load applications synchronously for now
        if let Some(apps) = load_from_cache() {
            launcher.applications = apps;
            launcher.fuzzy_matcher.set_candidates(&launcher.applications);
            launcher.update_search_results();
        }

        launcher
    }

    fn update_search_results(&mut self) {
        // Cancel previous search if it exists
        self.search_task = None;

        let query_str = self.query.to_string();
        self.search_results.clear();

        if is_process_query(&query_str) {
            let processes = get_running_processes();
            if query_str == "ps" {
                for process in processes {
                    self.search_results.push(SearchResult::Process(process));
                }
            } else if query_str.starts_with("ps") && query_str.len() > 2 {
                let search_term = query_str.strip_prefix("ps").unwrap_or("").to_lowercase();
                for process in processes {
                    if process.name.to_lowercase().contains(&search_term) {
                        self.search_results.push(SearchResult::Process(process));
                    }
                }
            }
        } else if is_calculator_query(&query_str) {
            if let Some(calculator) = &mut self.calculator {
                if let Some(result) = calculator.evaluate(&query_str) {
                    self.search_results.push(SearchResult::Calculation(result));
                }
            } else {
                self.search_results.push(SearchResult::Calculation(
                    "Initializing calculator...".to_string(),
                ));
            }
        } else {
            let app_indices = if query_str.is_empty() {
                (0..self.applications.len()).collect()
            } else {
                self.fuzzy_matcher.search(&query_str)
            };

            for index in app_indices.into_iter().take(50) {
                self.search_results.push(SearchResult::Application(index));
            }
        }

        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        let mut query = self.query.to_string();
        if !query.is_empty() {
            query.pop();
            self.query = query.into();
            self.update_search_results();
            cx.notify();
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        if !self.search_results.is_empty() && self.selected_index > 0 {
            self.selected_index -= 1;
            if self.selected_index < self.scroll_offset {
                self.scroll_offset = self.selected_index;
            }
            cx.notify();
        }
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        if !self.search_results.is_empty() && self.selected_index + 1 < self.search_results.len() {
            self.selected_index += 1;
            let visible_items = 10;
            if self.selected_index >= self.scroll_offset + visible_items {
                self.scroll_offset = self.selected_index - visible_items + 1;
            }
            cx.notify();
        }
    }

    fn launch(&mut self, _: &Launch, _: &mut Window, cx: &mut Context<Self>) {
        let _query_str = self.query.to_string();

        if let Some(result) = self.search_results.get(self.selected_index) {
            match result {
                SearchResult::Application(app_index) => {
                    if let Some(app) = self.applications.get(*app_index) {
                        let exec = app.exec.clone();
                        let name = app.name.clone();

                        std::thread::spawn(move || {
                            let mut cmd = Command::new("sh");
                            cmd.arg("-c")
                                .arg(&exec)
                                .stdin(std::process::Stdio::null())
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null());

                            match cmd.spawn() {
                                Ok(_) => eprintln!("[nlauncher] Launched: {name}"),
                                Err(err) => eprintln!(
                                    "[nlauncher] Failed to launch {name} (exec: {exec}): {err}"
                                ),
                            }
                        });

                        cx.quit();
                    }
                }
                SearchResult::Calculation(result) => {
                    if result != "Initializing calculator..." {
                        match std::process::Command::new("wl-copy").arg(result).output() {
                            Ok(_) => eprintln!("[nlauncher] Copied to clipboard: {result}"),
                            Err(e) => eprintln!("[nlauncher] Failed to copy to clipboard: {e}"),
                        }
                        cx.quit();
                    }
                }
                SearchResult::Process(process) => {
                    let pid = process.pid;
                    let name = process.name.clone();
                    std::thread::spawn(move || match kill_process(pid) {
                        Ok(_) => eprintln!("[nlauncher] Killed process: {name} (pid: {pid})"),
                        Err(e) => {
                            eprintln!("[nlauncher] Failed to kill process {name} (pid: {pid}): {e}")
                        }
                    });

                    self.update_search_results();
                    if self.selected_index >= self.search_results.len() && self.selected_index > 0 {
                        self.selected_index = self.search_results.len().saturating_sub(1);
                    }
                    cx.notify();
                }
            }
        }
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_index = self.selected_index;
        let query_text = self.query.to_string();
        let focus_handle = self.focus_handle.clone();

        div()
            .track_focus(&focus_handle)
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "space" {
                    let query = this.query.to_string();

                    // Block space in ps mode
                    if query.starts_with("ps") {
                        return;
                    }

                    let mut query = query;
                    query.push(' ');

                    this.query = query.into();
                    this.update_search_results();
                    cx.notify();
                } else if let Some(key_char) = &event.keystroke.key_char {
                    let allowed = key_char.chars().all(|c| c.is_alphanumeric() || "+-*/()^.=".contains(c));

                    if allowed {
                        let mut query = this.query.to_string();
                        query.push_str(key_char);
                        this.query = query.into();
                        this.update_search_results();
                        cx.notify();
                    }
                }
            }))
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::up))
            .on_action(cx.listener(Self::down))
            .on_action(cx.listener(Self::launch))
            .child(
                div()
                    .w(px(700.))
                    .max_h(px(500.))
                    .bg(self.theme.bg.opacity(0.87))
                    .border_1()
                    .border_color(self.theme.accent.opacity(0.2))
                    .rounded_lg()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(
                        div()
                            .p_2()
                            .bg(self.theme.surface)
                            .rounded_md()
                            .flex()
                            .gap_1()
                            .text_color(if query_text.is_empty() {
                                self.theme.text_muted
                            } else {
                                self.theme.text
                            })
                            .child(if query_text.is_empty() {
                                div().child("Search for apps and commands")
                            } else if query_text.starts_with("ps") {
                                let (cmd, rest) = if query_text.starts_with("ps") && query_text.len() > 2 {
                                    ("ps".to_string(), query_text.strip_prefix("ps").unwrap_or("").to_string())
                                } else if query_text == "ps" {
                                    ("ps".to_string(), String::new())
                                } else {
                                    (String::new(), query_text.clone())
                                };

                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_1()
                                            .bg(self.theme.success)
                                            .text_color(self.theme.text)
                                            .rounded_sm()
                                            .child(cmd)
                                    )
                                    .child(rest)
                            } else if query_text.starts_with("pass") {
                                let (cmd, rest) = if query_text.starts_with("pass") && query_text.len() > 4 {
                                    ("pass".to_string(), query_text.strip_prefix("pass").unwrap_or("").to_string())
                                } else if query_text == "pass" {
                                    ("pass".to_string(), String::new())
                                } else {
                                    (String::new(), query_text.clone())
                                };

                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_1()
                                            .bg(self.theme.warning)
                                            .text_color(self.theme.text)
                                            .rounded_sm()
                                            .child(cmd)
                                    )
                                    .child(rest)
                            } else if query_text.starts_with("clip") {
                                let (cmd, rest) = if query_text.starts_with("clip") && query_text.len() > 4 {
                                    ("clip".to_string(), query_text.strip_prefix("clip").unwrap_or("").to_string())
                                } else if query_text == "clip" {
                                    ("clip".to_string(), String::new())
                                } else {
                                    (String::new(), query_text.clone())
                                };

                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_1()
                                            .bg(self.theme.accent)
                                            .text_color(self.theme.text)
                                            .rounded_sm()
                                            .child(cmd)
                                    )
                                    .child(rest)
                            } else if query_text.starts_with('=') {
                                let rest = query_text.strip_prefix('=').unwrap_or("").to_string();
                                div()
                                    .flex()
                                    .gap_1()
                                    .child(
                                        div()
                                            .px_1()
                                            .bg(self.theme.accent_alt)
                                            .text_color(self.theme.text)
                                            .rounded_sm()
                                            .child("=")
                                    )
                                    .child(rest)
                            } else {
                                div().child(query_text.clone())
                            }),
                    )
                    .child(div().flex().flex_col().mt_2().children({
                        let visible_items = 10;
                        self.search_results
                            .iter()
                            .enumerate()
                            .skip(self.scroll_offset)
                            .take(visible_items)
                            .map(|(original_index, result)| {
                                let mut item = div()
                                    .flex()
                                    .items_center()
                                    .p_2()
                                    .text_color(self.theme.text_muted)
                                    .rounded_md()
                                    .hover(|style| style.bg(self.theme.overlay));

                                if original_index == selected_index {
                                    item = item.bg(self.theme.accent.opacity(0.2)).text_color(self.theme.accent);
                                }

                                match result {
                                    SearchResult::Application(app_index) => {
                                        if let Some(app) = self.applications.get(*app_index) {
                                            item.child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(
                                                        if let Some(icon_path) = &app.icon_path {
                                                            div().size_6().child(
                                                                img(std::path::PathBuf::from(
                                                                    icon_path,
                                                                ))
                                                                .size_6(),
                                                            )
                                                        } else {
                                                            div()
                                                                .size_6()
                                                                .bg(self.theme.accent_alt)
                                                                .rounded_sm()
                                                        },
                                                    )
                                                    .child(app.name.clone()),
                                            )
                                        } else {
                                            item.child("Invalid app")
                                        }
                                    }
                                    SearchResult::Calculation(calc_result) => item.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .size_6()
                                                    .bg(self.theme.success)
                                                    .rounded_sm()
                                                    .child("="),
                                            )
                                            .child(format!("= {calc_result}")),
                                    ),
                                    SearchResult::Process(process) => item.child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .size_6()
                                                    .bg(self.theme.error)
                                                    .rounded_sm()
                                                    .child("⚡"),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(format!(
                                                        "{} ({})",
                                                        process.name, process.pid
                                                    ))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(self.theme.accent)
                                                            .child(format!(
                                                                "CPU: {:.1}% | RAM: {:.1}MB",
                                                                process.cpu_usage,
                                                                process.memory_mb
                                                            )),
                                                    ),
                                            ),
                                    ),
                                }
                            })
                    })),
            )
    }
}

fn get_lock_path() -> PathBuf {
    let runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("nlauncher.lock")
}

fn is_running() -> bool {
    let lock_path = get_lock_path();
    if !lock_path.exists() {
        return false;
    }

    if let Ok(pid_str) = fs::read_to_string(&lock_path) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            return std::path::Path::new(&format!("/proc/{pid}")).exists();
        }
    }
    false
}

fn create_lock() {
    let lock_path = get_lock_path();
    let pid = std::process::id();
    let _ = fs::write(lock_path, pid.to_string());
}

fn remove_lock() {
    let lock_path = get_lock_path();
    let _ = fs::remove_file(lock_path);
}

fn main() {
    if is_running() {
        if let Ok(pid_str) = fs::read_to_string(get_lock_path()) {
            if let Ok(pid) = pid_str.trim().parse::<i32>() {
                unsafe {
                    libc::kill(pid, libc::SIGTERM);
                }
            }
        }
        std::process::exit(0);
    }

    create_lock();

    let _ = std::panic::catch_unwind(|| {
        Application::new().run(|cx: &mut App| {
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.bind_keys([
                KeyBinding::new("backspace", Backspace, None),
                KeyBinding::new("up", Up, None),
                KeyBinding::new("down", Down, None),
                KeyBinding::new("enter", Launch, None),
                KeyBinding::new("escape", Quit, None),
            ]);

            let window = cx
                .open_window(
                    WindowOptions {
                        titlebar: None,
                        window_bounds: Some(WindowBounds::Windowed(Bounds {
                            origin: point(px(0.), px(0.)),
                            size: Size::new(px(800.), px(600.)),
                        })),
                        app_id: Some("nlauncher".to_string()),
                        window_background: WindowBackgroundAppearance::Transparent,
                        kind: WindowKind::LayerShell(LayerShellOptions {
                            namespace: "nlauncher".to_string(),
                            anchor: Anchor::empty(),
                            margin: Some((px(0.), px(0.), px(0.), px(0.))),
                            keyboard_interactivity: KeyboardInteractivity::Exclusive,
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    |_, cx| cx.new(Launcher::new),
                )
                .unwrap();

            window
                .update(cx, |view, window, cx| {
                    window.focus(&view.focus_handle, cx);
                    cx.activate(true);
                })
                .unwrap();
        });
    });

    remove_lock();
}

pub struct LauncherWidget {
    launcher: Launcher,
}

impl LauncherWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            launcher: Launcher::new_for_widget(cx),
        }
    }
}

impl Render for LauncherWidget {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Delegate to launcher's render logic but adapt the context
        let selected_index = self.launcher.selected_index;
        let query_text = self.launcher.query.to_string();
        let focus_handle = self.launcher.focus_handle.clone();

        div()
            .track_focus(&focus_handle)
            .size_full()
            .bg(self.launcher.theme.bg)
            .child(
                div()
                    .p_4()
                    .child(format!("Launcher - Query: {}", query_text))
                    .child(format!("Selected: {}", selected_index))
                    .text_color(self.launcher.theme.text)
            )
    }
}
