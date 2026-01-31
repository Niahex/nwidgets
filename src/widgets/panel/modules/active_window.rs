use makepad_widgets::*;
use std::sync::Arc;

use crate::HYPRLAND_SERVICE;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    AppIcon = {{AppIcon}} {
        width: 24, height: 24
        
        draw_icon: {
            svg_file: dep("crate://self/assets/none.svg")
            fn get_color(self) -> vec4 {
                return #fff
            }
        }
    }

    pub ActiveWindowModule = {{ActiveWindowModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.0, y: 0.5}
        spacing: 8

        icon = <AppIcon> {}

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
pub struct AppIcon {
    #[redraw]
    #[live]
    draw_icon: DrawIcon,
    
    #[walk]
    walk: Walk,
}

impl Widget for AppIcon {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}
    
    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_icon.draw_walk(cx, walk);
        DrawStep::done()
    }
}

impl AppIcon {
    pub fn set_icon_path(&mut self, path: &str) {
        self.draw_icon.svg_path = ArcStringMut::String(path.to_string());
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
    timer: Timer,
}

impl Widget for ActiveWindowModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(0.5);
        }

        if let Event::Startup = event {
            ::log::info!("ActiveWindowModule: Startup event received");
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(0.5);
        }

        self.view.handle_event(cx, event, scope);
    }
}

impl ActiveWindowModule {
    fn get_icon_path(class: &str) -> &'static str {
        if class.is_empty() {
            return "crate://self/assets/none.svg";
        }
        
        let class_lower = class.to_lowercase();
        
        match class_lower.as_str() {
            "kitty" => "crate://self/assets/kitty.svg",
            "firefox" => "crate://self/assets/firefox-white.svg",
            "brave-browser" | "brave" => "crate://self/assets/brave.svg",
            "discord" => "crate://self/assets/discord.svg",
            "spotify" => "crate://self/assets/spotify.svg",
            "code" | "code-oss" | "vscode" | "zed" => "crate://self/assets/dev.zed.zed.svg",
            "vlc" => "crate://self/assets/vlc.svg",
            "steam" => "crate://self/assets/steam_tray.svg",
            "lutris" => "crate://self/assets/lutris.svg",
            "org.gnome.nautilus" | "nautilus" => "crate://self/assets/org.gnome.nautilus.svg",
            "org.inkscape.inkscape" | "inkscape" => "crate://self/assets/org.inkscape.inkscape.svg",
            "org.keepassxc.keepassxc" | "keepassxc" => "crate://self/assets/org.keepassxc.keepassxc.svg",
            "element" => "crate://self/assets/element.svg",
            "neochat" => "crate://self/assets/neochat.svg",
            "calibre-gui" | "calibre" => "crate://self/assets/calibre-gui.svg",
            "qbittorrent" => "crate://self/assets/qbittorrent.svg",
            "resolve" | "davinci-resolve" => "crate://self/assets/resolve.svg",
            "twitch" => "crate://self/assets/twitch.svg",
            "zen-twilight" => "crate://self/assets/zen-twilight.svg",
            "libreoffice-writer" => "crate://self/assets/libreoffice-writer.svg",
            "libreoffice-calc" => "crate://self/assets/libreoffice-calc.svg",
            "libreoffice-draw" => "crate://self/assets/libreoffice-draw.svg",
            "libreoffice-math" => "crate://self/assets/libreoffice-math.svg",
            _ => "crate://self/assets/none.svg",
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
            ::log::info!("ActiveWindowModule: class='{}' -> icon path={}", window.class, icon_path);
            
            if let Some(mut app_icon) = self.view.app_icon(ids!(icon)).borrow_mut() {
                app_icon.set_icon_path(icon_path);
            }
            cx.redraw_all();
        }
    }
}
