use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::shaders::*;

    pub NwIconBase = {{NwIcon}} {
        draw_icon: {
            svg_path: ""
        }
    }

    pub NwIcon = <NwIconBase> {
        width: Fit,
        height: Fit,
        
        icon_walk: {
            width: 20.0,
            height: Fit,
        }
        
        draw_bg: {
            fn pixel(self) -> vec4 {
                return vec4(0.0, 0.0, 0.0, 0.0)
            }
        }

        draw_icon: {
            uniform brightness: 1.0
            uniform curve: 0.6
            
            fn get_color(self) -> vec4 {
                return self.color
            }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct NwIcon {
    #[redraw]
    #[live]
    draw_bg: DrawQuad,
    
    #[live]
    pub draw_icon: DrawIcon,
    
    #[live]
    icon_walk: Walk,
    
    #[walk]
    walk: Walk,
    
    #[layout]
    layout: Layout,
}

impl Widget for NwIcon {
    fn handle_event(&mut self, _cx: &mut Cx, _event: &Event, _scope: &mut Scope) {}

    fn draw_walk(&mut self, cx: &mut Cx2d, _scope: &mut Scope, walk: Walk) -> DrawStep {
        self.draw_bg.begin(cx, walk, self.layout);
        self.draw_icon.draw_walk(cx, self.icon_walk);
        self.draw_bg.end(cx);
        DrawStep::done()
    }
}

impl NwIcon {
    pub fn set_svg_path(&mut self, cx: &mut Cx, path: &str) {
        self.draw_icon.svg_path.as_mut_empty().push_str(path);
        self.draw_bg.redraw(cx);
    }
    
    pub fn set_color(&mut self, color: Vec4) {
        self.draw_icon.color = color;
    }
    
    pub fn set_brightness(&mut self, brightness: f32) {
        self.draw_icon.brightness = brightness;
    }
    
    pub fn set_scale(&mut self, scale: f64) {
        self.draw_icon.scale = scale;
    }
}

