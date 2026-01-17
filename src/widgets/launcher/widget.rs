use gpui::{
    actions, background_executor, div, layer_shell::*, point, prelude::*, px, App,
    Application, Bounds, Context, FocusHandle, KeyBinding, KeyDownEvent, Render,
    Size, Task, Window, WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions,
};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::components::{SearchInput, SearchResults, SearchResult};
use crate::services::{applications, calculator, process, launcher::LauncherService};
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

enum SearchResultType {
    Application(usize),
    Calculation(String),
    Process(ProcessInfo),
}

struct Launcher {
    focus_handle: FocusHandle,
    applications: Vec<ApplicationInfo>,
    search_input: SearchInput,
    search_results: SearchResults,
    fuzzy_matcher: FuzzyMatcher,
    calculator: Option<Calculator>,
    internal_results: Vec<SearchResultType>,
    search_task: Option<Task<()>>,
    theme: Theme,
}

impl Launcher {
    fn new(cx: &mut Context<Self>) -> Self {
        let theme = Theme::nord_dark();
        let mut launcher = Self {
            focus_handle: cx.focus_handle(),
            applications: Vec::new(),
            search_input: SearchInput::new("Search for apps and commands").with_theme(theme.clone()),
            search_results: SearchResults::new().with_theme(theme.clone()),
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: Some(Calculator::new()),
            internal_results: Vec::new(),
            search_task: None,
            theme,
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

        launcher
    }

    fn new_for_widget<T>(cx: &mut Context<T>) -> Self {
        let theme = Theme::nord_dark();
        let mut launcher = Self {
            focus_handle: cx.focus_handle(),
            applications: Vec::new(),
            search_input: SearchInput::new("Search for apps and commands").with_theme(theme.clone()),
            search_results: SearchResults::new().with_theme(theme.clone()),
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: Some(Calculator::new()),
            internal_results: Vec::new(),
            search_task: None,
            theme,
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

        let query_str = self.search_input.get_query();
        self.internal_results.clear();

        if is_process_query(query_str) {
            let processes = get_running_processes();
            if query_str == "ps" {
                for process in processes {
                    self.internal_results.push(SearchResultType::Process(process));
                }
            } else if query_str.starts_with("ps") && query_str.len() > 2 {
                let search_term = query_str.strip_prefix("ps").unwrap_or("").to_lowercase();
                for process in processes {
                    if process.name.to_lowercase().contains(&search_term) {
                        self.internal_results.push(SearchResultType::Process(process));
                    }
                }
            }
        } else if is_calculator_query(query_str) {
            if let Some(calculator) = &mut self.calculator {
                if let Some(result) = calculator.evaluate(query_str) {
                    self.internal_results.push(SearchResultType::Calculation(result));
                }
            } else {
                self.internal_results.push(SearchResultType::Calculation(
                    "Initializing calculator...".to_string(),
                ));
            }
        } else {
            let app_indices = if query_str.is_empty() {
                (0..self.applications.len()).collect()
            } else {
                self.fuzzy_matcher.search(query_str)
            };

            for index in app_indices.into_iter().take(50) {
                self.internal_results.push(SearchResultType::Application(index));
            }
        }

        // Convert internal results to SearchResult for the component
        let display_results: Vec<SearchResult> = self.internal_results.iter().map(|result| {
            match result {
                SearchResultType::Application(index) => {
                    if let Some(app) = self.applications.get(*index) {
                        SearchResult::Application(app.clone())
                    } else {
                        SearchResult::Application(ApplicationInfo {
                            name: "Invalid app".to_string(),
                            name_lower: "invalid app".to_string(),
                            exec: "".to_string(),
                            icon: None,
                            icon_path: None,
                        })
                    }
                }
                SearchResultType::Calculation(calc) => SearchResult::Calculation(calc.clone()),
                SearchResultType::Process(proc) => SearchResult::Process(proc.clone()),
            }
        }).collect();

        self.search_results.set_results(display_results);
    }

    fn backspace(&mut self, _: &Backspace, _: &mut Window, cx: &mut Context<Self>) {
        let mut query = self.search_input.get_query().to_string();
        if !query.is_empty() {
            query.pop();
            self.search_input.set_query(query);
            self.update_search_results();
            cx.notify();
        }
    }

    fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.search_results.move_selection_up();
        cx.notify();
    }

    fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.search_results.move_selection_down();
        cx.notify();
    }

    fn launch(&mut self, _: &Launch, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(selected_result) = self.search_results.get_selected() {
            match selected_result {
                SearchResult::Application(app) => {
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
                    cx.notify();
                }
            }
        }
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle.clone();

        div()
            .track_focus(&focus_handle)
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "space" {
                    let query = this.search_input.get_query().to_string();

                    // Block space in ps mode
                    if query.starts_with("ps") {
                        return;
                    }

                    let mut query = query;
                    query.push(' ');

                    this.search_input.set_query(query);
                    this.update_search_results();
                    cx.notify();
                } else if let Some(key_char) = &event.keystroke.key_char {
                    let allowed = key_char.chars().all(|c| c.is_alphanumeric() || "+-*/()^.=".contains(c));

                    if allowed {
                        let mut query = this.search_input.get_query().to_string();
                        query.push_str(key_char);
                        this.search_input.set_query(query);
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
                    .child(self.search_input.render_with_handlers(
                        |_query| {}, // Input handling is done in key_down
                        |_query| {}, // Space handling is done in key_down
                    ))
                    .child(self.search_results.render()),
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
    launcher_service: gpui::Entity<LauncherService>,
}

impl LauncherWidget {
    pub fn new(cx: &mut Context<Self>, launcher_service: gpui::Entity<LauncherService>) -> Self {
        Self {
            launcher: Launcher::new_for_widget(cx),
            launcher_service,
        }
    }

    pub fn focus_handle(&self) -> &FocusHandle {
        &self.launcher.focus_handle
    }

    pub fn reset(&mut self) {
        self.launcher.search_input.set_query("");
        self.launcher.update_search_results();
    }
}

impl Render for LauncherWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.launcher.focus_handle.clone();

        div()
            .track_focus(&focus_handle)
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                if event.keystroke.key == "space" {
                    let query = this.launcher.search_input.get_query().to_string();

                    // Block space in ps mode
                    if query.starts_with("ps") {
                        return;
                    }

                    let mut query = query;
                    query.push(' ');

                    this.launcher.search_input.set_query(query);
                    this.launcher.update_search_results();
                    cx.notify();
                } else if let Some(key_char) = &event.keystroke.key_char {
                    let allowed = key_char.chars().all(|c| c.is_alphanumeric() || "+-*/()^.=".contains(c));

                    if allowed {
                        let mut query = this.launcher.search_input.get_query().to_string();
                        query.push_str(key_char);
                        this.launcher.search_input.set_query(query);
                        this.launcher.update_search_results();
                        cx.notify();
                    }
                }
            }))
            .on_action(cx.listener(|this, _: &Backspace, _window, cx| {
                let mut query = this.launcher.search_input.get_query().to_string();
                if !query.is_empty() {
                    query.pop();
                    this.launcher.search_input.set_query(query);
                    this.launcher.update_search_results();
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &Up, _window, cx| {
                this.launcher.search_results.move_selection_up();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Down, _window, cx| {
                this.launcher.search_results.move_selection_down();
                cx.notify();
            }))
            .on_action(cx.listener(|this, _: &Quit, _window, cx| {
                // Hide the launcher
                this.launcher_service.update(cx, |service, cx| {
                    service.toggle(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &Launch, _window, cx| {
                if let Some(selected_result) = this.launcher.search_results.get_selected() {
                    match selected_result {
                        SearchResult::Application(app) => {
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
                                    Ok(_) => eprintln!("[launcher] Launched: {name}"),
                                    Err(err) => eprintln!(
                                        "[launcher] Failed to launch {name} (exec: {exec}): {err}"
                                    ),
                                }
                            });

                            // Hide launcher after launch
                            this.launcher_service.update(cx, |service, cx| {
                                service.toggle(cx);
                            });
                        }
                        SearchResult::Calculation(result) => {
                            if result != "Initializing calculator..." {
                                match std::process::Command::new("wl-copy").arg(result).output() {
                                    Ok(_) => eprintln!("[launcher] Copied to clipboard: {result}"),
                                    Err(e) => eprintln!("[launcher] Failed to copy to clipboard: {e}"),
                                }
                                // Hide launcher after copy
                                this.launcher_service.update(cx, |service, cx| {
                                    service.toggle(cx);
                                });
                            }
                        }
                        SearchResult::Process(process) => {
                            let pid = process.pid;
                            let name = process.name.clone();
                            std::thread::spawn(move || match kill_process(pid) {
                                Ok(_) => eprintln!("[launcher] Killed process: {name} (pid: {pid})"),
                                Err(e) => {
                                    eprintln!("[launcher] Failed to kill process {name} (pid: {pid}): {e}")
                                }
                            });

                            this.launcher.update_search_results();
                            cx.notify();
                        }
                    }
                }
            }))
            .child(
                div()
                    .w(px(700.))
                    .max_h(px(500.))
                    .bg(self.launcher.theme.bg.opacity(0.87))
                    .border_1()
                    .border_color(self.launcher.theme.accent.opacity(0.2))
                    .rounded_lg()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(self.launcher.search_input.render_with_handlers(
                        |_query| {},
                        |_query| {},
                    ))
                    .child(self.launcher.search_results.render()),
            )
    }
}
