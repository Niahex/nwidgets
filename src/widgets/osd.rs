use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
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

        flow: Row
        align: {x: 0.5, y: 0.5}
        padding: 16
        spacing: 16

        visible: false

        icon = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 24.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
            text: "󰕾"
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
        }
    }
}

#[derive(Live, LiveHook, Widget, WidgetRef)]
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
    pub fn show_volume(&mut self, cx: &mut Cx, volume: f32, muted: bool) {
        self.osd_type = OSDType::Volume;
        self.value = volume;

        let icon = if muted {
            "󰖁"
        } else if volume > 0.66 {
            "󰕾"
        } else if volume > 0.33 {
            "󰖀"
        } else if volume > 0.0 {
            "󰕿"
        } else {
            "󰝟"
        };

        self.view.label(ids!(icon)).set_text(cx, icon);
        self.view.apply_over(cx, live! { 
            visible: true
            progress_bar = {
                draw_bg: {
                    value: (volume)
                }
            }
        });

        self.hide_timer = cx.start_timeout(2.0);
        self.view.redraw(cx);
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

        self.view.label(ids!(icon)).set_text(cx, "󰅎");
        self.view.apply_over(cx, live! { 
            visible: true
            progress_bar = {
                visible: false
            }
        });

        let _ = text;

        self.hide_timer = cx.start_timeout(2.0);
        self.view.redraw(cx);
    }

    fn hide(&mut self, cx: &mut Cx) {
        self.view.apply_over(cx, live! { visible: false });
        self.view.redraw(cx);
    }
}


