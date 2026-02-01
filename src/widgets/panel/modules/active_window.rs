use makepad_widgets::*;
use std::sync::{Arc, Mutex};

use crate::HYPRLAND_SERVICE;
use crate::ui::components::icon::NwIconWidgetExt;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;
    use crate::ui::components::*;

    AppIcon = <NwIcon> {
        width: 24, height: 24
        icon_walk: { width: 24, height: 24 }
        draw_icon: {
            brightness: 1.0
            curve: 0.6
            color: #fff
        }
    }

    pub ActiveWindowModule = {{ActiveWindowModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.0, y: 0.5}
        spacing: 8

        icon = <AppIcon> {
            draw_icon: {
                svg_file: dep("crate://self/assets/icons/none.svg")
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
            return "crate://self/assets/icons/none.svg";
        }
        
        let class_lower = class.to_lowercase();
        
        match class_lower.as_str() {
            "kitty" => "crate://self/assets/icons/kitty.svg",
            "firefox" => "crate://self/assets/icons/firefox-white.svg",
            "brave-browser" | "brave" => "crate://self/assets/icons/brave.svg",
            "discord" => "crate://self/assets/icons/discord.svg",
            "spotify" => "crate://self/assets/icons/spotify.svg",
            "code" | "code-oss" | "vscode" | "zed" => "crate://self/assets/icons/dev.zed.zed.svg",
            "vlc" => "crate://self/assets/icons/vlc.svg",
            "steam" => "crate://self/assets/icons/steam_tray.svg",
            "lutris" => "crate://self/assets/icons/lutris.svg",
            "org.gnome.nautilus" | "nautilus" => "crate://self/assets/icons/org.gnome.nautilus.svg",
            "org.inkscape.inkscape" | "inkscape" => "crate://self/assets/icons/org.inkscape.inkscape.svg",
            "org.keepassxc.keepassxc" | "keepassxc" => "crate://self/assets/icons/org.keepassxc.keepassxc.svg",
            "element" => "crate://self/assets/icons/element.svg",
            "neochat" => "crate://self/assets/icons/neochat.svg",
            "calibre-gui" | "calibre" => "crate://self/assets/icons/calibre-gui.svg",
            "qbittorrent" => "crate://self/assets/icons/qbittorrent.svg",
            "resolve" | "davinci-resolve" => "crate://self/assets/icons/resolve.svg",
            "twitch" => "crate://self/assets/icons/twitch.svg",
            "zen-twilight" => "crate://self/assets/icons/zen-twilight.svg",
            "libreoffice-writer" => "crate://self/assets/icons/libreoffice-writer.svg",
            "libreoffice-calc" => "crate://self/assets/icons/libreoffice-calc.svg",
            "libreoffice-draw" => "crate://self/assets/icons/libreoffice-draw.svg",
            "libreoffice-math" => "crate://self/assets/icons/libreoffice-math.svg",
            _ => "crate://self/assets/icons/none.svg",
        }
    }

    fn sync_from_service(&mut self, cx: &mut Cx) {
        let window = HYPRLAND_SERVICE.get_active_window();
        
        if window.title != self.window_title || window.class != self.window_class {
            self.window_title = window.title.clone();
            self.window_class = window.class.clone();
            
            let title = if window.title.is_empty() {
                "No window"
            } else {
                &window.title
            };
            
            self.view.label(ids!(info.title)).set_text(cx, title);
            self.view.label(ids!(info.class)).set_text(cx, &window.class);
            
            let icon_path = Self::get_icon_path(&window.class);
            ::log::info!("ActiveWindowModule: class='{}' -> icon_path={}", window.class, icon_path);
            
            if let Some(mut icon) = self.view.nw_icon(ids!(icon)).borrow_mut() {
                icon.draw_icon.svg_path = ArcStringMut::String(icon_path.to_string());
            }
            
            cx.redraw_all();
        }
    }
}
