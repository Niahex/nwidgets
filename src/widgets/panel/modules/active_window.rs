use makepad_widgets::*;
use std::sync::{Arc, Mutex};

use crate::HYPRLAND_SERVICE;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    AppIconNone = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/none.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconKitty = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/kitty.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconFirefox = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/firefox-white.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconBrave = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/brave.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconDiscord = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/discord.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconSpotify = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/spotify.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconZed = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/dev.zed.zed.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconVlc = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/vlc.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }
    
    AppIconSteam = <Icon> {
        width: 24, height: 24
        draw_icon: {
            svg_file: dep("crate://self/assets/icons/steam_tray.svg")
            fn get_color(self) -> vec4 { return #fff }
        }
    }

    pub ActiveWindowModule = {{ActiveWindowModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.0, y: 0.5}
        spacing: 8

        icon_container = <View> {
            width: 24, height: 24
            
            icon_none = <AppIconNone> { visible: true }
            icon_kitty = <AppIconKitty> { visible: false }
            icon_firefox = <AppIconFirefox> { visible: false }
            icon_brave = <AppIconBrave> { visible: false }
            icon_discord = <AppIconDiscord> { visible: false }
            icon_spotify = <AppIconSpotify> { visible: false }
            icon_zed = <AppIconZed> { visible: false }
            icon_vlc = <AppIconVlc> { visible: false }
            icon_steam = <AppIconSteam> { visible: false }
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
    fn get_icon_id(class: &str) -> &'static str {
        if class.is_empty() {
            return "icon_none";
        }
        
        let class_lower = class.to_lowercase();
        
        match class_lower.as_str() {
            "kitty" => "icon_kitty",
            "firefox" => "icon_firefox",
            "brave-browser" | "brave" => "icon_brave",
            "discord" => "icon_discord",
            "spotify" => "icon_spotify",
            "code" | "code-oss" | "vscode" | "zed" => "icon_zed",
            "vlc" => "icon_vlc",
            "steam" => "icon_steam",
            _ => "icon_none",
        }
    }
    
    fn hide_all_icons(&mut self, cx: &mut Cx) {
        self.view.view(ids!(icon_container.icon_none)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_kitty)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_firefox)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_brave)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_discord)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_spotify)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_zed)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_vlc)).set_visible(cx, false);
        self.view.view(ids!(icon_container.icon_steam)).set_visible(cx, false);
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
            
            let icon_id = Self::get_icon_id(&window.class);
            ::log::info!("ActiveWindowModule: class='{}' -> icon_id={}", window.class, icon_id);
            
            self.hide_all_icons(cx);
            
            match icon_id {
                "icon_none" => self.view.view(ids!(icon_container.icon_none)).set_visible(cx, true),
                "icon_kitty" => self.view.view(ids!(icon_container.icon_kitty)).set_visible(cx, true),
                "icon_firefox" => self.view.view(ids!(icon_container.icon_firefox)).set_visible(cx, true),
                "icon_brave" => self.view.view(ids!(icon_container.icon_brave)).set_visible(cx, true),
                "icon_discord" => self.view.view(ids!(icon_container.icon_discord)).set_visible(cx, true),
                "icon_spotify" => self.view.view(ids!(icon_container.icon_spotify)).set_visible(cx, true),
                "icon_zed" => self.view.view(ids!(icon_container.icon_zed)).set_visible(cx, true),
                "icon_vlc" => self.view.view(ids!(icon_container.icon_vlc)).set_visible(cx, true),
                "icon_steam" => self.view.view(ids!(icon_container.icon_steam)).set_visible(cx, true),
                _ => self.view.view(ids!(icon_container.icon_none)).set_visible(cx, true),
            }
            
            cx.redraw_all();
        }
    }
}
