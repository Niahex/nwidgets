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

        icon = <Image> {
            width: 32, height: 32
            source: dep("crate://self/assets/icons/png/help.png")
        }

        info = <View> {
            width: Fit, height: Fit
            flow: Down
            spacing: 2

            class = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: (THEME_COLOR_TEXT_MUTE) }
                text: ""
            }

            title = <Label> {
                draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
                text: "No window"
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
    fn get_icon_path(class: &str) -> String {
        if class.is_empty() {
            return "crate://self/assets/icons/png/help.png".to_string();
        }
        
        let normalized_class = class.to_lowercase().replace('.', "-");
        let icon_filename = format!("{}.png", normalized_class);
        let icon_path = format!("crate://self/assets/icons/png/{}", icon_filename);
        let fs_path = format!("./assets/icons/png/{}", icon_filename);
        
        if std::path::Path::new(&fs_path).exists() {
            icon_path
        } else {
            "crate://self/assets/icons/png/help.png".to_string()
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

            if let Some(mut image) = self.view.image(ids!(icon)).borrow_mut() {
            }

            self.view.redraw(cx);
        }
    }
}
