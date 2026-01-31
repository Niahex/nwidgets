use makepad_widgets::*;

use crate::HYPRLAND_SERVICE;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use makepad_draw::shader::std::*;
    use crate::theme::*;

    WorkspaceIndicator = <View> {
        width: 8, height: 8

        show_bg: true
        draw_bg: {
            instance active: 0.0
            instance occupied: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                let center = self.rect_size * 0.5;
                let radius = min(center.x, center.y);

                let color = mix(
                    mix(#4C566A, #88C0D0, self.occupied),
                    #88C0D0,
                    self.active
                );

                sdf.circle(center.x, center.y, radius);
                sdf.fill(color);

                return sdf.result;
            }
        }
    }

    pub WorkspacesModule = {{WorkspacesModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.5, y: 0.5}
        spacing: 6
        padding: {left: 8, right: 8}

        ws1 = <WorkspaceIndicator> {}
        ws2 = <WorkspaceIndicator> {}
        ws3 = <WorkspaceIndicator> {}
        ws4 = <WorkspaceIndicator> {}
        ws5 = <WorkspaceIndicator> {}
        ws6 = <WorkspaceIndicator> {}
        ws7 = <WorkspaceIndicator> {}
        ws8 = <WorkspaceIndicator> {}
        ws9 = <WorkspaceIndicator> {}
        ws10 = <WorkspaceIndicator> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct WorkspacesModule {
    #[deref]
    view: View,

    #[rust]
    active_workspace: i32,

    #[rust]
    occupied_workspaces: Vec<i32>,
    
    #[rust]
    timer: Timer,
}

impl Widget for WorkspacesModule {
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }

    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if self.timer.is_event(event).is_some() {
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(0.5);
        }

        if let Event::Startup = event {
            ::log::info!("WorkspacesModule: Startup event received");
            self.sync_from_service(cx);
            self.timer = cx.start_timeout(0.5);
        }

        self.view.handle_event(cx, event, scope);
    }
}

impl WorkspacesModule {
    fn sync_from_service(&mut self, cx: &mut Cx) {
        let active = HYPRLAND_SERVICE.get_active_workspace();
        let occupied = HYPRLAND_SERVICE.get_occupied_workspaces();
        
        let occupied_vec: Vec<i32> = occupied.iter().copied().collect();
        
        if active != self.active_workspace || occupied_vec != self.occupied_workspaces {
            self.active_workspace = active;
            self.occupied_workspaces = occupied_vec;
            
            ::log::info!("WorkspacesModule: active={}, occupied={:?}", active, self.occupied_workspaces);
            
            for i in 1..=10 {
                let ws_id = match i {
                    1 => ids!(ws1),
                    2 => ids!(ws2),
                    3 => ids!(ws3),
                    4 => ids!(ws4),
                    5 => ids!(ws5),
                    6 => ids!(ws6),
                    7 => ids!(ws7),
                    8 => ids!(ws8),
                    9 => ids!(ws9),
                    10 => ids!(ws10),
                    _ => continue,
                };
                
                let is_active = i == active;
                let is_occupied = self.occupied_workspaces.contains(&i);
                
                self.view.view(ws_id).apply_over(cx, live!{
                    draw_bg: {
                        active: (if is_active { 1.0 } else { 0.0 })
                        occupied: (if is_occupied { 1.0 } else { 0.0 })
                    }
                });
            }
            
            cx.redraw_all();
        }
    }
    
    pub fn set_active(&mut self, _cx: &mut Cx, workspace: i32) {
        self.active_workspace = workspace;
    }

    pub fn set_occupied(&mut self, _cx: &mut Cx, workspaces: Vec<i32>) {
        self.occupied_workspaces = workspaces;
    }
}
