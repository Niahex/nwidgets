use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use makepad_draw::shader::std::*;
    use crate::theme::*;

    pub OSD = {{OSD}} {
        width: 300, height: 80

        show_bg: true
        draw_bg: {
            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, 16.0);
                sdf.fill((NORD_POLAR_0));
                return sdf.result;
            }
        }

        flow: Right
        align: {x: 0.5, y: 0.5}
        padding: 16
        spacing: 16

        visible: false

        icon = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 24.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
            text: "󰕾"
        }
        
        capslock_icon = <Icon> {
            width: 0, height: 0
            icon_walk: { width: 0, height: 0 }
            draw_icon: {
                svg_file: dep("crate://self/assets/icons/capslock-off.svg")
                brightness: 1.0
                curve: 0.6
                color: #fff
                preserve_colors: true
            }
        }
        
        text_label = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 14.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
            text: ""
        }

        progress_bar = <View> {
            width: Fill, height: 8

            show_bg: true
            draw_bg: {
                instance value: 0.5

                fn pixel(self) -> vec4 {
                    let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                    let radius = self.rect_size.y * 0.5;

                    sdf.box(0.0, 0.0, self.rect_size.x, self.rect_size.y, radius);
                    sdf.fill((NORD_POLAR_3));

                    let filled = self.rect_size.x * self.value;
                    sdf.box(0.0, 0.0, filled, self.rect_size.y, radius);
                    sdf.fill((NORD_FROST_1));

                    return sdf.result;
                }
            }
            visible: true
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct OSD {
    #[deref]
    view: View,

    #[rust]
    hide_timer: Timer,

    #[rust]
    osd_type: OSDType,

    #[rust]
    value: f32,
}

#[derive(Clone, Debug, Default)]
pub enum OSDType {
    #[default]
    Volume,
    Brightness,
    Clipboard,
    CapsLock,
}

impl Widget for OSD {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.hide_timer.is_event(event).is_some() {
            self.hide(cx);
        }

        self.view.handle_event(cx, event, scope);
    }
}

impl OSD {
    fn get_volume_icon_path(volume: f32, muted: bool) -> &'static str {
        if muted {
            "./assets/icons/sink-muted.svg"
        } else if volume == 0.0 {
            "./assets/icons/sink-zero.svg"
        } else if volume > 0.66 {
            "./assets/icons/sink-high.svg"
        } else if volume > 0.33 {
            "./assets/icons/sink-medium.svg"
        } else {
            "./assets/icons/sink-low.svg"
        }
    }
    
    pub fn show_volume(&mut self, cx: &mut Cx, volume: f32, muted: bool) {
        self.osd_type = OSDType::Volume;
        self.value = volume;

        let icon_path = Self::get_volume_icon_path(volume, muted);

        ::log::info!("OSD: Setting volume icon to {} and volume to {}", icon_path, volume);
        
        if let Some(mut icon) = self.view.icon(ids!(capslock_icon)).borrow_mut() {
            icon.set_icon_from_path(cx, icon_path);
        }
        
        self.view.label(ids!(icon)).set_text(cx, "");
        self.view.label(ids!(text_label)).set_text(cx, "");
        
        self.view.apply_over(cx, live! { 
            visible: true
            capslock_icon = {
                width: 32, height: 32
                icon_walk: { width: 32, height: 32 }
            }
            progress_bar = {
                visible: true
                draw_bg: {
                    value: (volume)
                }
            }
        });

        self.hide_timer = cx.start_timeout(2.0);
        self.view.redraw(cx);
        cx.redraw_all();
        
        ::log::info!("OSD: show_volume completed");
    }

    pub fn show_brightness(&mut self, cx: &mut Cx, brightness: f32) {
        self.osd_type = OSDType::Brightness;
        self.value = brightness;

        let icon = if brightness > 0.66 {
            "󰃠"
        } else if brightness > 0.33 {
            "󰃟"
        } else {
            "󰃞"
        };

        self.view.label(ids!(icon)).set_text(cx, icon);
        self.view.apply_over(cx, live! { visible: true });

        self.hide_timer = cx.start_timeout(2.0);
        self.view.redraw(cx);
    }

    pub fn show_clipboard(&mut self, cx: &mut Cx, text: &str) {
        self.osd_type = OSDType::Clipboard;

        let icon_path = "./assets/icons/clipboard.svg";
        
        ::log::info!("OSD: Clipboard copied: {} bytes", text.len());
        
        if let Some(mut icon) = self.view.icon(ids!(capslock_icon)).borrow_mut() {
            icon.set_icon_from_path(cx, icon_path);
        }
        
        self.view.label(ids!(icon)).set_text(cx, "");
        self.view.label(ids!(text_label)).set_text(cx, "");
        
        self.view.apply_over(cx, live! { 
            visible: true
            capslock_icon = {
                width: 32, height: 32
                icon_walk: { width: 32, height: 32 }
            }
            progress_bar = {
                visible: false
            }
        });

        let _ = text;

        self.hide_timer = cx.start_timeout(2.0);
        self.view.redraw(cx);
        cx.redraw_all();
    }

    pub fn show_capslock(&mut self, cx: &mut Cx, enabled: bool) {
        self.osd_type = OSDType::CapsLock;

        let icon_path = if enabled {
            "./assets/icons/capslock-on.svg"
        } else {
            "./assets/icons/capslock-off.svg"
        };
        
        let text = if enabled { "Caps Lock ON" } else { "Caps Lock OFF" };

        ::log::info!("OSD: Caps Lock {} - icon: {}", if enabled { "enabled" } else { "disabled" }, icon_path);

        if let Some(mut icon) = self.view.icon(ids!(capslock_icon)).borrow_mut() {
            icon.set_icon_from_path(cx, icon_path);
        }
        
        self.view.label(ids!(icon)).set_text(cx, "");
        self.view.label(ids!(text_label)).set_text(cx, text);
        
        self.view.apply_over(cx, live! { 
            visible: true
            capslock_icon = {
                width: 32, height: 32
                icon_walk: { width: 32, height: 32 }
            }
            progress_bar = {
                visible: false
            }
        });

        self.hide_timer = cx.start_timeout(2.0);
        self.view.redraw(cx);
        cx.redraw_all();

        ::log::info!("OSD: show_capslock completed");
    }

    fn hide(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: false });
        self.view.redraw(cx);
    }
}


