use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};
use gpui_component::list::{List, ListDelegate, ListEvent, ListState};
use gpui_component::{Icon, IndexPath, Selectable, Sizable};
use nwidgets_service_applications::{AppInfo, ApplicationsService, ApplicationsStateChanged};
use nwidgets_service_clipboard::{ClipboardChanged, ClipboardEntry, ClipboardService};
use nwidgets_service_process::{ProcessInfo, kill_process, search_processes};
use std::process::Command;

const CORNER_RADIUS: f32 = 12.0;

actions!(launcher, [CloseLauncher]);

// ── Mode du launcher ──────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum LauncherMode {
    Apps,
    Processes,
    Clipboard,
}

// ── Item de la liste (App ou Processus) ──────────────────────────────────────

#[derive(Clone)]
pub enum LauncherEntry {
    App(AppInfo),
    Process(ProcessInfo),
    Clipboard(ClipboardEntry),
}

#[derive(IntoElement, Clone)]
pub struct LauncherListItem {
    entry: LauncherEntry,
    selected: bool,
}

impl Selectable for LauncherListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }
    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for LauncherListItem {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let frost0: Hsla  = rgb(0xd8dee9).into();
        let muted: Hsla   = rgb(0x4c566a).into();
        let accent: Hsla  = rgb(0x88c0d0).into();
        let red: Hsla     = rgb(0xbf616a).into();
        let selected_bg: Hsla = rgb(0x434c5e).into();
        let hover_bg: Hsla    = Hsla { h: frost0.h, s: frost0.s, l: frost0.l, a: 0.08 };
        let border: Hsla = if self.selected { accent } else { rgb(0x000000).opacity(0.0).into() };

        let group = match &self.entry {
            LauncherEntry::App(a)     => format!("item-app-{}", a.name),
            LauncherEntry::Process(p) => format!("item-proc-{}", p.pid),
            LauncherEntry::Clipboard(e) => format!("item-clip-{}", e.timestamp.timestamp_millis()),
        };

        match self.entry {
            LauncherEntry::App(app) => div()
                .group(group.clone())
                .flex()
                .items_center()
                .gap_3()
                .px_3()
                .py_2()
                .when(self.selected, |d| d.bg(selected_bg))
                .when(!self.selected, |d| d.group_hover(&group, |s| s.bg(hover_bg)))
                .border_1()
                .border_color(border)
                .rounded_md()
                .child(
                    if let Some(ref path) = app.icon_path {
                        div().size(px(24.0)).flex_shrink_0()
                            .child(img(std::path::PathBuf::from(path)).size(px(24.0)))
                    } else {
                        div().size(px(24.0)).flex_shrink_0()
                            .flex().items_center().justify_center()
                            .child(Icon::new("apps").size(px(20.0)).text_color(accent))
                    },
                )
                .child(
                    div().flex().flex_col()
                        .child(
                            div().text_sm().font_weight(FontWeight::BOLD)
                                .text_color(frost0).child(app.name.clone()),
                        )
                        .child(
                            div().text_xs().text_color(muted)
                                .group_hover(&group, |s| s.text_color(frost0))
                                .child({
                                    let exec = &app.exec;
                                    if exec.len() > 40 { format!("{}…", &exec[..40]) } else { exec.clone() }
                                }),
                        ),
                ),

            LauncherEntry::Process(proc) => div()
                .group(group.clone())
                .flex()
                .items_center()
                .gap_3()
                .px_3()
                .py_2()
                .when(self.selected, |d| d.bg(selected_bg))
                .when(!self.selected, |d| d.group_hover(&group, |s| s.bg(hover_bg)))
                .border_1()
                .border_color(border)
                .rounded_md()
                .child(
                    div().size(px(24.0)).flex_shrink_0()
                        .flex().items_center().justify_center()
                        .child(Icon::new("memory").size(px(20.0)).text_color(red)),
                )
                .child(
                    div().flex().flex_col()
                        .child(
                            div().text_sm().font_weight(FontWeight::BOLD)
                                .text_color(frost0)
                                .child(format!("{} (PID {})", proc.name, proc.pid)),
                        )
                        .child(
                            div().text_xs().text_color(muted)
                                .group_hover(&group, |s| s.text_color(frost0))
                                .child(format!("CPU {:.1}%  RAM {:.1}MB", proc.cpu_usage, proc.memory_mb)),
                        ),
                ),

            LauncherEntry::Clipboard(entry) => {
                let preview = entry.content.replace(['\n', '\r'], " ");
                let ts = entry.timestamp.format("%H:%M:%S").to_string();
                let truncated = if preview.len() > 52 {
                    format!("{}…", &preview[..52])
                } else {
                    preview
                };

                div()
                    .group(group.clone())
                    .flex().items_center().gap_3().px_3().py_2()
                    .when(self.selected, |d| d.bg(selected_bg))
                    .when(!self.selected, |d| d.group_hover(&group, |s| s.bg(hover_bg)))
                    .border_1().border_color(border).rounded_md()
                    .child(
                        div().size(px(24.0)).flex_shrink_0()
                            .flex().items_center().justify_center()
                            .child(Icon::new("content_paste").size(px(20.0)).text_color(accent)),
                    )
                    .child(
                        div().flex().flex_col()
                            .child(
                                div().text_sm().font_weight(FontWeight::BOLD)
                                    .text_color(frost0).child(truncated),
                            )
                            .child(
                                div().text_xs().text_color(muted)
                                    .group_hover(&group, |s| s.text_color(frost0))
                                    .child(ts),
                            ),
                    )
            }
        }
    }
}

// ── ListDelegate ──────────────────────────────────────────────────────────────

pub struct LauncherDelegate {
    pub all_apps: Vec<AppInfo>,
    pub entries: Vec<LauncherEntry>,
    pub mode: LauncherMode,
    pub selected_index: Option<IndexPath>,
    pub last_confirmed: Option<LauncherEntry>,
    pub clipboard_history: Vec<ClipboardEntry>,
}

impl LauncherDelegate {
    pub fn new(apps: Vec<AppInfo>) -> Self {
        let entries = apps.iter().cloned().map(LauncherEntry::App).collect();
        Self {
            all_apps: apps,
            entries,
            mode: LauncherMode::Apps,
            selected_index: None,
            last_confirmed: None,
            clipboard_history: Vec::new(),
        }
    }

    pub fn update_clipboard(&mut self, history: Vec<ClipboardEntry>, cx: &mut Context<ListState<Self>>) {
        self.clipboard_history = history;
        if self.mode == LauncherMode::Clipboard {
            self.entries = self.clipboard_history.iter().cloned().map(LauncherEntry::Clipboard).collect();
            cx.notify();
        }
    }

    pub fn update_apps(&mut self, apps: Vec<AppInfo>, cx: &mut Context<ListState<Self>>) {
        self.all_apps = apps.clone();
        if self.mode == LauncherMode::Apps {
            self.entries = apps.into_iter().map(LauncherEntry::App).collect();
            self.selected_index = None;
            cx.notify();
        }
    }
}

impl ListDelegate for LauncherDelegate {
    type Item = LauncherListItem;

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> gpui::Task<()> {
        if query.starts_with("clip") {
            // Mode Clipboard
            self.mode = LauncherMode::Clipboard;
            let term = query.strip_prefix("clip").unwrap_or("").trim().to_lowercase();
            self.entries = self.clipboard_history.iter()
                .filter(|e| term.is_empty() || e.content.to_lowercase().contains(&term))
                .cloned()
                .map(LauncherEntry::Clipboard)
                .collect();
        } else if query.starts_with("ps") {
            // Mode Processus
            self.mode = LauncherMode::Processes;
            let procs = search_processes(query);
            self.entries = procs.into_iter().map(LauncherEntry::Process).collect();
        } else {
            // Mode Applications
            self.mode = LauncherMode::Apps;
            if query.trim().is_empty() {
                self.entries = self.all_apps.iter().cloned().map(LauncherEntry::App).collect();
            } else {
                let q = query.to_lowercase();
                self.entries = self.all_apps.iter()
                    .filter(|a| a.name.to_lowercase().contains(&q) || a.exec.to_lowercase().contains(&q))
                    .cloned()
                    .map(LauncherEntry::App)
                    .collect();
            }
        }

        self.selected_index = if self.entries.is_empty() { None } else { Some(IndexPath::new(0)) };
        cx.notify();
        gpui::Task::ready(())
    }

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.entries.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let entry = self.entries.get(ix.row)?.clone();
        let selected = self.selected_index == Some(ix);
        Some(LauncherListItem { entry, selected })
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }

    fn confirm(
        &mut self,
        _secondary: bool,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        if let Some(ix) = &self.selected_index {
            if let Some(entry) = self.entries.get(ix.row).cloned() {
                self.last_confirmed = Some(entry);
                cx.notify();
            }
        }
    }
}

// ── Launcher View ─────────────────────────────────────────────────────────────

pub struct Launcher {
    pub focus_handle: FocusHandle,
    apps_service: Entity<ApplicationsService>,
    clipboard_service: Entity<ClipboardService>,
    list_state: Entity<ListState<LauncherDelegate>>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<CloseLauncher> for Launcher {}

impl Launcher {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let apps_service = ApplicationsService::global(cx);
        let clipboard_service = ClipboardService::global(cx);

        let initial_apps = apps_service.read(cx).applications.clone();
        let initial_clipboard = clipboard_service.read(cx).history.iter().cloned().collect();

        let mut delegate = LauncherDelegate::new(initial_apps);
        delegate.clipboard_history = initial_clipboard;
        let list_state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

        let mut subscriptions = Vec::new();

        // Reload apps when service updates
        let ls = list_state.clone();
        subscriptions.push(cx.subscribe(
            &apps_service,
            move |this, _, _: &ApplicationsStateChanged, cx| {
                let new_apps = this.apps_service.read(cx).applications.clone();
                ls.update(cx, |list, cx| list.delegate_mut().update_apps(new_apps, cx));
            },
        ));

        // Reload clipboard history when it changes
        let ls_clip = list_state.clone();
        subscriptions.push(cx.subscribe(
            &clipboard_service,
            move |this, _, _: &ClipboardChanged, cx| {
                let history = this.clipboard_service.read(cx).history.iter().cloned().collect();
                ls_clip.update(cx, |list, cx| list.delegate_mut().update_clipboard(history, cx));
            },
        ));

        // Confirm: lance app, kill process ou copie clipboard
        let ls2 = list_state.clone();
        subscriptions.push(cx.subscribe(
            &ls2,
            |this, _, ev: &ListEvent, cx| {
                if let ListEvent::Confirm(_) = ev {
                    let confirmed = this.list_state.read(cx).delegate().last_confirmed.clone();
                    match confirmed {
                        Some(LauncherEntry::App(app)) => {
                            let exec = app.exec.clone();
                            std::thread::spawn(move || {
                                let _ = Command::new("sh").arg("-c").arg(exec).spawn();
                            });
                            cx.emit(CloseLauncher);
                        }
                        Some(LauncherEntry::Process(proc)) => {
                            let pid = proc.pid;
                            std::thread::spawn(move || { let _ = kill_process(pid); });
                        }
                        Some(LauncherEntry::Clipboard(entry)) => {
                            ClipboardService::copy_to_clipboard(&entry.content);
                            cx.emit(CloseLauncher);
                        }
                        None => {}
                    }
                }
            },
        ));

        Self { focus_handle, apps_service, clipboard_service, list_state, _subscriptions: subscriptions }
    }

    /// Vide le champ de recherche du launcher
    pub fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |list, cx| {
            list.set_query("", window, cx);
        });
    }
}

impl Render for Launcher {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);

        // Force focus vers la barre de recherche
        let search_fh = self.list_state.focus_handle(cx);
        window.focus(&search_fh, cx);

        div()
            .id("launcher-main")
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _action: &CloseLauncher, window, cx| {
                this.reset(window, cx);
                cx.emit(CloseLauncher);
            }))
            .size_full()
            .flex()
            .flex_row()
            // ── Left Concave Corner ──
            .child(
                div().h_full().w(px(CORNER_RADIUS)).flex().flex_col()
                    .child(Corner::new(CornerPosition::TopRight, px(CORNER_RADIUS)).color(bg))
                    .child(div().flex_1()),
            )
            // ── Launcher Body ──
            .child(
                div()
                    .w_full().size_full().bg(bg).rounded_b(px(CORNER_RADIUS))
                    .flex().flex_col().p_3()
                    .child(List::new(&self.list_state).with_size(gpui_component::Size::Medium)),
            )
            // ── Right Concave Corner ──
            .child(
                div().h_full().w(px(CORNER_RADIUS)).flex().flex_col()
                    .child(Corner::new(CornerPosition::TopLeft, px(CORNER_RADIUS)).color(bg))
                    .child(div().flex_1()),
            )
    }
}
