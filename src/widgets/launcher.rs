use gpui::{
    actions, div, prelude::*, px, Animation, AnimationExt, Context, FocusHandle, KeyDownEvent,
    Render, Task, Window,
};
use std::time::Duration;

use crate::components::{SearchInput, SearchResult, SearchResults};
use crate::services::clipboard::ClipboardMonitor;
use crate::services::launcher::{
    process, LauncherCore, LauncherService, LauncherToggled, SearchResultType,
};
use crate::theme::Theme;
use process::kill_process;

actions!(launcher, [Quit, Backspace, Up, Down, Launch, OpenSettings]);

struct Launcher {
    focus_handle: FocusHandle,
    core: LauncherCore,
    search_input: SearchInput,
    search_results: SearchResults,
    internal_results: Vec<SearchResultType>,
    search_task: Option<Task<()>>,
    theme: Theme,
}

impl Launcher {
    fn new_for_widget<T>(cx: &mut Context<T>) -> Self {
        let theme = Theme::nord_dark();
        let mut core = LauncherCore::new();
        core.load_from_cache();

        Self {
            focus_handle: cx.focus_handle(),
            core,
            search_input: SearchInput::new("Search for apps and commands").with_theme(&theme),
            search_results: SearchResults::new().with_theme(&theme),
            internal_results: Vec::new(),
            search_task: None,
            theme,
        }
    }

    fn update_search_results(&mut self) {
        self.update_search_results_with_clipboard(Vec::new());
    }

    fn update_search_results_with_clipboard(&mut self, clipboard_history: Vec<String>) {
        self.search_task = None;

        let query_str = self.search_input.get_query();
        self.internal_results = self.core.search(query_str, clipboard_history);

        let display_results = self.core.convert_to_display_results(&self.internal_results);
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
                    let mut exec = app.exec.clone();
                    let name = app.name.clone();

                    // Remove desktop entry field codes
                    exec = exec
                        .replace("%U", "")
                        .replace("%u", "")
                        .replace("%F", "")
                        .replace("%f", "")
                        .replace("%i", "")
                        .replace("%c", "")
                        .replace("%k", "");
                    let exec = exec.trim().to_string();

                    // Spawn in tokio runtime, not GPUI executor
                    let _ = tokio::spawn(async move {
                        log::info!("Launching application: {name} with command: {exec}");

                        use std::os::unix::process::CommandExt;

                        // Double fork to completely detach
                        unsafe {
                            let result = std::process::Command::new("sh")
                                .arg("-c")
                                .arg(&exec)
                                .env_remove("LD_LIBRARY_PATH")
                                .stdin(std::process::Stdio::null())
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .pre_exec(|| {
                                    // Create new session
                                    if libc::setsid() == -1 {
                                        return Err(std::io::Error::last_os_error());
                                    }
                                    Ok(())
                                })
                                .spawn();

                            match result {
                                Ok(_) => log::info!("Successfully launched: {name}"),
                                Err(err) => log::error!("Failed to launch {name}: {err}"),
                            }
                        }
                    });

                    cx.quit();
                }
                SearchResult::Calculation(result) => {
                    if result != "Initializing calculator..." {
                        let result = result.clone();
                        cx.background_executor()
                            .spawn(async move {
                                match std::process::Command::new("wl-copy").arg(&result).output() {
                                    Ok(_) => log::info!("Copied to clipboard: {result}"),
                                    Err(e) => log::error!("Failed to copy to clipboard: {e}"),
                                }
                            })
                            .detach();
                        cx.quit();
                    }
                }
                SearchResult::Process(process) => {
                    let pid = process.pid;
                    let name = process.name.clone();
                    cx.background_executor()
                        .spawn(async move {
                            match kill_process(pid) {
                                Ok(_) => log::info!("Killed process: {name} (pid: {pid})"),
                                Err(e) => {
                                    log::error!("Failed to kill process {name} (pid: {pid}): {e}")
                                }
                            }
                        })
                        .detach();

                    self.update_search_results();
                    cx.notify();
                }
                SearchResult::Clipboard(_) => {
                    // Not used in standalone Launcher
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
                    let allowed = key_char
                        .chars()
                        .all(|c| c.is_alphanumeric() || "+-*/()^.=".contains(c));

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
                    .bg(self.theme.bg)
                    .border_1()
                    .border_color(self.theme.accent_alt.opacity(0.25))
                    .rounded(px(12.))
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

pub struct LauncherWidget {
    launcher: Launcher,
    launcher_service: gpui::Entity<LauncherService>,
    clipboard_monitor: gpui::Entity<ClipboardMonitor>,
    visible: bool,
}

impl LauncherWidget {
    pub fn new(
        cx: &mut Context<Self>,
        launcher_service: gpui::Entity<LauncherService>,
        clipboard_monitor: gpui::Entity<ClipboardMonitor>,
    ) -> Self {
        let launcher = Launcher::new_for_widget(cx);

        // Scan apps in background (async, non-blocking)
        let apps_arc = launcher.core.applications.clone();
        cx.spawn(|this: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                // Scan sur le background executor (thread pool)
                let apps = cx
                    .background_executor()
                    .spawn(async { crate::services::launcher::applications::scan_applications() })
                    .await;

                // Update UI thread
                let _ = this.update(&mut cx, |this, cx| {
                    *apps_arc.write() = apps.clone();
                    this.launcher
                        .core
                        .fuzzy_matcher
                        .set_candidates(&apps_arc.read());
                    cx.notify();
                });

                // Save cache in background
                cx.background_executor()
                    .spawn(async move {
                        let _ = crate::services::launcher::applications::save_to_cache(&apps);
                    })
                    .detach();
            }
        })
        .detach();

        // Écouter les changements de visibilité
        cx.subscribe(
            &launcher_service,
            |this, _service, _event: &LauncherToggled, cx| {
                this.visible = this.launcher_service.read(cx).visible;
                cx.notify();
            },
        )
        .detach();

        let initial_visible = launcher_service.read(cx).visible;

        Self {
            launcher,
            launcher_service,
            clipboard_monitor,
            visible: initial_visible,
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
        // Si pas visible, retourner un élément vide
        if !self.visible {
            return div().into_any_element();
        }

        let focus_handle = self.launcher.focus_handle.clone();

        div()
            .id("launcher-root")
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
                    let clipboard_history = this.clipboard_monitor.read(cx).get_history();
                    this.launcher
                        .update_search_results_with_clipboard(clipboard_history);
                    cx.notify();
                } else if let Some(key_char) = &event.keystroke.key_char {
                    let allowed = key_char
                        .chars()
                        .all(|c| c.is_alphanumeric() || "+-*/()^.=".contains(c));

                    if allowed {
                        let mut query = this.launcher.search_input.get_query().to_string();
                        query.push_str(key_char);
                        this.launcher.search_input.set_query(query);
                        let clipboard_history = this.clipboard_monitor.read(cx).get_history();
                        this.launcher
                            .update_search_results_with_clipboard(clipboard_history);
                        cx.notify();
                    }
                }
            }))
            .on_action(cx.listener(|this, _: &Backspace, _window, cx| {
                let mut query = this.launcher.search_input.get_query().to_string();
                if !query.is_empty() {
                    query.pop();
                    this.launcher.search_input.set_query(query);
                    let clipboard_history = this.clipboard_monitor.read(cx).get_history();
                    this.launcher
                        .update_search_results_with_clipboard(clipboard_history);
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
                            let mut exec = app.exec.clone();
                            let name = app.name.clone();

                            // Remove desktop entry field codes
                            exec = exec
                                .replace("%U", "")
                                .replace("%u", "")
                                .replace("%F", "")
                                .replace("%f", "")
                                .replace("%i", "")
                                .replace("%c", "")
                                .replace("%k", "");
                            let exec = exec.trim().to_string();

                            cx.background_executor()
                                .spawn(async move {
                                    log::info!("Launching: {name} with command: '{exec}'");

                                    use std::os::unix::process::CommandExt;
                                    unsafe {
                                        let result = std::process::Command::new("sh")
                                            .arg("-c")
                                            .arg(&exec)
                                            .env_remove("LD_LIBRARY_PATH")
                                            .pre_exec(|| {
                                                if libc::setsid() == -1 {
                                                    return Err(std::io::Error::last_os_error());
                                                }
                                                Ok(())
                                            })
                                            .spawn();

                                        match result {
                                            Ok(_) => log::info!("Successfully spawned: {name}"),
                                            Err(err) => {
                                                log::error!("Failed to spawn {name}: {err}")
                                            }
                                        }
                                    }
                                })
                                .detach();

                            // Hide launcher after launch
                            this.launcher_service.update(cx, |service, cx| {
                                service.toggle(cx);
                            });
                        }
                        SearchResult::Calculation(result) => {
                            if result != "Initializing calculator..." {
                                let result = result.clone();
                                cx.background_executor()
                                    .spawn(async move {
                                        match std::process::Command::new("wl-copy")
                                            .arg(&result)
                                            .output()
                                        {
                                            Ok(_) => log::info!("Copied to clipboard: {result}"),
                                            Err(e) => {
                                                log::error!("Failed to copy to clipboard: {e}")
                                            }
                                        }
                                    })
                                    .detach();
                                // Hide launcher after copy
                                this.launcher_service.update(cx, |service, cx| {
                                    service.toggle(cx);
                                });
                            }
                        }
                        SearchResult::Process(process) => {
                            let pid = process.pid;
                            let name = process.name.clone();
                            cx.background_executor()
                                .spawn(async move {
                                    match kill_process(pid) {
                                        Ok(_) => log::info!("Killed process: {name} (pid: {pid})"),
                                        Err(e) => log::error!(
                                            "Failed to kill process {name} (pid: {pid}): {e}"
                                        ),
                                    }
                                })
                                .detach();

                            let clipboard_history = this.clipboard_monitor.read(cx).get_history();
                            this.launcher
                                .update_search_results_with_clipboard(clipboard_history);
                            cx.notify();
                        }
                        SearchResult::Clipboard(content) => {
                            let content = content.clone();
                            cx.background_executor()
                                .spawn(async move {
                                    match std::process::Command::new("wl-copy")
                                        .arg(&content)
                                        .output()
                                    {
                                        Ok(_) => log::info!("Copied clipboard entry"),
                                        Err(e) => log::error!("Failed to copy: {e}"),
                                    }
                                })
                                .detach();
                            // Hide launcher after copy
                            this.launcher_service.update(cx, |service, cx| {
                                service.toggle(cx);
                            });
                        }
                    }
                }
            }))
            .child(
                div()
                    .w(px(700.))
                    .max_h(px(500.))
                    .bg(self.launcher.theme.bg)
                    .border_1()
                    .border_color(self.launcher.theme.accent_alt.opacity(0.25))
                    .rounded(px(12.))
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(
                        self.launcher
                            .search_input
                            .render_with_handlers(|_query| {}, |_query| {}),
                    )
                    .child(self.launcher.search_results.render())
                    .with_animation(
                        "launcher-fade-in",
                        Animation::new(Duration::from_millis(150)),
                        |this, delta| this.opacity(delta),
                    ),
            )
            .into_any_element()
    }
}
