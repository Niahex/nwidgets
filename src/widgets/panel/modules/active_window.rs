use makepad_widgets::*;
use std::sync::{Arc, Mutex};

use crate::HYPRLAND_SERVICE;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub ActiveWindowModule = {{ActiveWindowModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.0, y: 0.5}
        spacing: 8

        icon = <Icon> {
            width: 32, height: 32
            icon_walk: { width: 32, height: 32 }
            draw_icon: {
                svg_file: dep("crate://self/assets/icons/none.svg")
                brightness: 1.0
                curve: 0.6
                color: #fff
                preserve_colors: true
            }
        }

        info = <View> {
            width: Fit, height: Fit
            flow: Down
            spacing: 2

            title = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
                text: "No window"
            }

            class = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: (THEME_COLOR_TEXT_MUTE) }
                text: ""
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct ActiveWindowModule {
    #[deref]
    view: View,

    #[rust]
    window_title: String,

    #[rust]
    window_class: String,

    #[rust]
    needs_redraw: Arc<Mutex<bool>>,
    
    #[rust]
    timer: Timer,
}

impl Widget for ActiveWindowModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        if *self.needs_redraw.lock().unwrap() {
            *self.needs_redraw.lock().unwrap() = false;
            self.sync_from_service(cx);
        }
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            if *self.needs_redraw.lock().unwrap() {
                *self.needs_redraw.lock().unwrap() = false;
                self.sync_from_service(cx);
            }
            self.timer = cx.start_timeout(0.016);
        }

        if let Event::Startup = event {
            ::log::info!("ActiveWindowModule: Startup event received");
            self.sync_from_service(cx);
            
            let needs_redraw = self.needs_redraw.clone();
            HYPRLAND_SERVICE.on_change(move || {
                *needs_redraw.lock().unwrap() = true;
            });
            
            self.timer = cx.start_timeout(0.016);
        }

        self.view.handle_event(cx, event, scope);
    }
}

impl ActiveWindowModule {
    fn get_icon_path(class: &str) -> &'static str {
        if class.is_empty() {
            return "./assets/icons/none.svg";
        }
        
        let class_lower = class.to_lowercase();
        
        match class_lower.as_str() {
            "kitty" => "./assets/icons/kitty.svg",
            "firefox" => "./assets/icons/firefox-white.svg",
            "brave-browser" | "brave" => "./assets/icons/brave.svg",
            "discord" => "./assets/icons/discord.svg",
            "spotify" => "./assets/icons/spotify.svg",
            "code" | "code-oss" | "vscode" | "zed" | "dev.zed.zed" => "./assets/icons/dev.zed.zed.svg",
            "vlc" => "./assets/icons/vlc.svg",
            "steam" => "./assets/icons/steam_tray.svg",
            "lutris" => "./assets/icons/lutris.svg",
            "org.gnome.nautilus" | "nautilus" => "./assets/icons/org.gnome.nautilus.svg",
            "org.inkscape.inkscape" | "inkscape" => "./assets/icons/org.inkscape.inkscape.svg",
            "org.keepassxc.keepassxc" | "keepassxc" => "./assets/icons/org.keepassxc.keepassxc.svg",
            "element" => "./assets/icons/element.svg",
            "neochat" => "./assets/icons/neochat.svg",
            "calibre-gui" | "calibre" => "./assets/icons/calibre-gui.svg",
            "qbittorrent" => "./assets/icons/qbittorrent.svg",
            "resolve" | "davinci-resolve" => "./assets/icons/resolve.svg",
            "twitch" => "./assets/icons/twitch.svg",
            "zen-twilight" => "./assets/icons/zen-twilight.svg",
            "libreoffice-writer" => "./assets/icons/libreoffice-writer.svg",
            "libreoffice-calc" => "./assets/icons/libreoffice-calc.svg",
            "libreoffice-draw" => "./assets/icons/libreoffice-draw.svg",
            "libreoffice-math" => "./assets/icons/libreoffice-math.svg",
            _ => "./assets/icons/none.svg",
        }
    }

    fn sync_from_service(&mut self, cx: &mut Cx) {
        let window = HYPRLAND_SERVICE.get_active_window();
        let title = window.title.clone();
        let class = window.class.clone();

        if self.window_title != title || self.window_class != class {
            self.window_title = title.clone();
            self.window_class = class.clone();

            self.view.label(ids!(info.title)).set_text(cx, &title);
            self.view.label(ids!(info.class)).set_text(cx, &class);

            let icon_path = Self::get_icon_path(&class);
            ::log::info!("ActiveWindowModule: class='{}' -> icon_path={}", class, icon_path);

            if let Some(mut icon) = self.view.icon(ids!(icon)).borrow_mut() {
                icon.set_icon_from_path(cx, icon_path);
            }

            self.view.redraw(cx);
        }
    }
}
