use gpui::{
    actions, App, Application, Bounds, Context, FocusHandle, KeyBinding, KeyDownEvent, Render,
    SharedString, Size, Window, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions, div, img, layer_shell::*, point, prelude::*, px, rgb,
};
use std::process::Command;

mod applications;
mod state;
use applications::load_applications;
use state::ApplicationInfo;

actions!(launcher, [Quit, Backspace, Up, Down, Launch]);

struct Launcher {
    focus_handle: FocusHandle,
    applications: Vec<ApplicationInfo>,
    query: SharedString,
    query_lower: String,
    selected_index: usize,
    filtered_cache: Option<Vec<usize>>,
    last_query: String,
}

impl Launcher {
    fn new(cx: &mut Context<Self>) -> Self {
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
            
            // Lancer en arrière-plan avec environnement propre
            std::thread::spawn(move || {
                let mut cmd = Command::new("sh");
                cmd.arg("-c")
                   .arg(&exec)
                   .env_clear()  // Nettoyer l'environnement
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

    fn filtered_apps(&mut self) -> Vec<&ApplicationInfo> {
        // Early return pour query vide
        if self.query.is_empty() {
            self.filtered_cache = None;
            self.last_query.clear();
            return self.applications.iter().collect();
        }
        
        // Vérifier si on doit recalculer le cache
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
        
        // Retourner les résultats depuis le cache
        if let Some(ref indices) = self.filtered_cache {
            indices.iter().map(|&i| &self.applications[i]).collect()
        } else {
            Vec::new()
        }
    }
}

impl Render for Launcher {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Extraire toutes les valeurs nécessaires AVANT filtered_apps
        let selected_index = self.selected_index;
        let query_text = self.query.to_string();
        let focus_handle = self.focus_handle.clone();
        
        // Maintenant on peut appeler filtered_apps
        let filtered_apps = self.filtered_apps();
        let start_index = if selected_index >= 10 { selected_index - 9 } else { 0 };

        div()
            .track_focus(&focus_handle)
            .size_full()
            .bg(rgb(0x2e3440))
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
            .child(
                div()
                    .flex()
                    .flex_col()
                    .p_4()
                    .child(
                        div()
                            .p_2()
                            .bg(rgb(0x3b4252))
                            .rounded_md()
                            .text_color(rgb(0xeceff4))
                            .child(format!("Search: {}", query_text))
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
                                            .text_color(rgb(0xeceff4));
                                        
                                        if i == selected_index {
                                            item = item.bg(rgb(0x88c0d0));
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
                                                            .bg(rgb(0x5e81ac))
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

fn main() {
    env_logger::init();
    
    Application::new().run(|cx: &mut App| {
        cx.on_action(|_: &Quit, cx| cx.quit());
        cx.bind_keys([
            KeyBinding::new("backspace", Backspace, None),
            KeyBinding::new("up", Up, None),
            KeyBinding::new("down", Down, None),
            KeyBinding::new("enter", Launch, None),
            KeyBinding::new("escape", Quit, None),
        ]);

        let window = cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(600.), px(400.)),
                })),
                app_id: Some("nlauncher".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "nlauncher".to_string(),
                    anchor: Anchor::BOTTOM,
                    margin: Some((px(0.), px(0.), px(50.), px(0.))),
                    keyboard_interactivity: KeyboardInteractivity::Exclusive,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(Launcher::new),
        ).unwrap();

        window.update(cx, |view, window, cx| {
            window.focus(&view.focus_handle);
            cx.activate(true);
        }).unwrap();
    });
}
