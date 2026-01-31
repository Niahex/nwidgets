use makepad_widgets::*;

use crate::HYPRLAND_SERVICE;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use makepad_draw::shader::std::*;
    use crate::theme::*;

    WorkspaceButton = <View> {
        width: Fit, height: Fit
        padding: {left: 8, right: 8, top: 4, bottom: 4}
        cursor: Hand

        show_bg: true
        draw_bg: {
            instance active: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                
                let color = mix(
                    #4C566A,
                    #88C0D0,
                    self.active
                );

                sdf.box(
                    1.0,
                    1.0,
                    self.rect_size.x - 2.0,
                    self.rect_size.y - 2.0,
                    2.0
                );
                sdf.fill(color);

                return sdf.result;
            }
        }

        label = <Label> {
            draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 11.0 }, color: #ECEFF4 }
            text: "1"
        }
    }

    pub WorkspacesModule = {{WorkspacesModule}} {
        width: Fit, height: Fill
        flow: Right
        align: {x: 0.5, y: 0.5}
        spacing: 4
        padding: {left: 8, right: 8}

        ws1 = <WorkspaceButton> {}
        ws2 = <WorkspaceButton> {}
        ws3 = <WorkspaceButton> {}
        ws4 = <WorkspaceButton> {}
        ws5 = <WorkspaceButton> {}
        ws6 = <WorkspaceButton> {}
        ws7 = <WorkspaceButton> {}
        ws8 = <WorkspaceButton> {}
        ws9 = <WorkspaceButton> {}
        ws10 = <WorkspaceButton> {}
    }
}

#[derive(Live, LiveHook, Widget)]
pub struct WorkspacesModule {
    #[deref]
    view: View,

    #[rust]
    active_workspace: i32,

    #[rust]
    workspace_count: usize,
    
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
        let workspaces = HYPRLAND_SERVICE.get_workspaces();
        
        if active != self.active_workspace || workspaces.len() != self.workspace_count {
            self.active_workspace = active;
            self.workspace_count = workspaces.len();
            
            ::log::info!("WorkspacesModule: active={}, workspaces={:?}", 
                active, 
                workspaces.iter().map(|w| format!("{}:{}", w.id, w.name)).collect::<Vec<_>>()
            );
            
            let ws_ids = [
                ids!(ws1), ids!(ws2), ids!(ws3), ids!(ws4), ids!(ws5),
                ids!(ws6), ids!(ws7), ids!(ws8), ids!(ws9), ids!(ws10),
            ];
            
            for (idx, ws_id) in ws_ids.iter().enumerate() {
                if let Some(ws) = workspaces.get(idx) {
                    let is_active = ws.id == active;
                    
                    let display_name = if ws.name.parse::<i32>().is_ok() {
                        ws.name.clone()
                    } else {
                        ws.name
                            .chars()
                            .next()
                            .unwrap_or('?')
                            .to_uppercase()
                            .to_string()
                    };
                    
                    self.view.view(*ws_id).apply_over(cx, live!{
                        visible: true
                        draw_bg: {
                            active: (if is_active { 1.0 } else { 0.0 })
                        }
                    });
                    self.view.view(*ws_id).label(ids!(label)).set_text(cx, &display_name);
                } else {
                    self.view.view(*ws_id).apply_over(cx, live!{
                        visible: false
                    });
                }
            }
            
            cx.redraw_all();
        }
    }
}
