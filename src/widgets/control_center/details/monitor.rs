use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    MonitorCard = <View> {
        width: 100, height: 100
        flow: Down
        align: {x: 0.5, y: 0.5}
        spacing: 8
        padding: 12

        show_bg: true
        draw_bg: { color: (NORD_POLAR_1), radius: 12.0 }

        label = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: (THEME_COLOR_TEXT_MUTE) }
            text: "CPU"
        }

        value = <Label> {
            draw_text: { text_style: <THEME_FONT_BOLD> { font_size: 18.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
            text: "0%"
        }

        subvalue = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 10.0 }, color: (THEME_COLOR_TEXT_MUTE) }
            text: ""
        }
    }

    pub SystemMonitor = {{SystemMonitor}} {
        width: Fill, height: Fit
        flow: Row
        spacing: 12
        align: {x: 0.5, y: 0.5}

        cpu_card = <MonitorCard> {
            label = { text: "CPU" }
        }

        gpu_card = <MonitorCard> {
            label = { text: "GPU" }
        }

        ram_card = <MonitorCard> {
            label = { text: "RAM" }
        }

        temp_card = <MonitorCard> {
            label = { text: "Temp" }
        }
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct SystemMonitor {
    #[deref]
    view: View,

    #[rust]
    cpu_usage: f32,

    #[rust]
    gpu_usage: f32,

    #[rust]
    ram_usage: f32,

    #[rust]
    temperature: f32,
}

impl Widget for SystemMonitor {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
    }
}

impl SystemMonitor {
    pub fn update_stats(&mut self, cx: &mut Cx, cpu: f32, gpu: f32, ram: f32, temp: f32) {
        self.cpu_usage = cpu;
        self.gpu_usage = gpu;
        self.ram_usage = ram;
        self.temperature = temp;

        self.view.label(ids!(cpu_card.value)).set_text(cx, &format!("{:.0}%", cpu));
        self.view.label(ids!(gpu_card.value)).set_text(cx, &format!("{:.0}%", gpu));
        self.view.label(ids!(ram_card.value)).set_text(cx, &format!("{:.0}%", ram));
        self.view.label(ids!(temp_card.value)).set_text(cx, &format!("{:.0}°C", temp));
    }
}
