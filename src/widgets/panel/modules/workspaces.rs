use makepad_widgets::*;
use std::sync::{Arc, Mutex};

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
        align: {x: 0.5, y: 0.5}

        show_bg: true
        draw_bg: {
            instance active: 0.0
            instance hover: 0.0

            fn pixel(self) -> vec4 {
                let sdf = Sdf2d::viewport(self.pos * self.rect_size);
                
                let bg_color = mix(
                    vec4(0.0, 0.0, 0.0, 0.0),
                    mix(
                        (THEME_COLOR_TEXT_MUTE),
                        (COLOR_ACCENT),
                        self.active
                    ),
                    max(self.hover, self.active)
                );

                sdf.box(
                    1.0,
                    1.0,
                    self.rect_size.x - 2.0,
                    self.rect_size.y - 2.0,
                    4.0
                );
                sdf.fill(bg_color);

                return sdf.result;
            }
        }

        label = <Label> {
            draw_text: {
                text_style: <THEME_FONT_REGULAR> { font_size: 11.0 }
                
                instance active: 0.0
                instance hover: 0.0
                
                fn get_color(self) -> vec4 {
                    return mix(
                        (THEME_COLOR_TEXT_MUTE),
                        mix(
                            (THEME_COLOR_BG_APP),
                            (THEME_COLOR_ACCENT_ALT),
                            self.active
                        ),
                        max(self.hover, self.active)
                    );
                }
            }
            text: ""
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
    workspace_ids: [Option<i32>; 10],
    
    #[rust]
    hovered_workspace: Option<usize>,
    
    #[rust]
    needs_redraw: Arc<Mutex<bool>>,
    
    #[rust]
    timer: Timer,
}

impl Widget for WorkspacesModule {
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
            ::log::info!("WorkspacesModule: Startup event received");
            self.sync_from_service(cx);
            
            let needs_redraw = self.needs_redraw.clone();
            HYPRLAND_SERVICE.on_change(move || {
                *needs_redraw.lock().unwrap() = true;
            });
            
            self.timer = cx.start_timeout(0.016);
        }

        self.view.handle_event(cx, event, scope);

        let ws_ids = [
            ids!(ws1), ids!(ws2), ids!(ws3), ids!(ws4), ids!(ws5),
            ids!(ws6), ids!(ws7), ids!(ws8), ids!(ws9), ids!(ws10),
        ];

        for (idx, ws_id) in ws_ids.iter().enumerate() {
            if let Some(workspace_id) = self.workspace_ids[idx] {
                let button_view = self.view.view(*ws_id);
                match event.hits(cx, button_view.area()) {
                    Hit::FingerDown(_) => {
                        ::log::info!("Switching to workspace {}", workspace_id);
                        HYPRLAND_SERVICE.switch_workspace(workspace_id);
                    }
                    Hit::FingerHoverIn(_) => {
                        self.hovered_workspace = Some(idx);
                        self.update_workspace_hover(cx, idx, true);
                    }
                    Hit::FingerHoverOut(_) => {
                        if self.hovered_workspace == Some(idx) {
                            self.hovered_workspace = None;
                            self.update_workspace_hover(cx, idx, false);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

impl WorkspacesModule {
    fn update_workspace_hover(&mut self, cx: &mut Cx, idx: usize, is_hovering: bool) {
        let ws_ids = [
            ids!(ws1), ids!(ws2), ids!(ws3), ids!(ws4), ids!(ws5),
            ids!(ws6), ids!(ws7), ids!(ws8), ids!(ws9), ids!(ws10),
        ];
        
        if let Some(ws_id) = ws_ids.get(idx) {
            if let Some(workspace_id) = self.workspace_ids[idx] {
                let is_active = workspace_id == self.active_workspace;
                
                self.view.view(*ws_id).apply_over(cx, live!{
                    draw_bg: {
                        hover: (if is_hovering { 1.0 } else { 0.0 })
                    }
                    label = {
                        draw_text: {
                            hover: (if is_hovering { 1.0 } else { 0.0 })
                        }
                    }
                });
                
                cx.redraw_all();
            }
        }
    }
    
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
            
            self.workspace_ids = [None; 10];
            
            for (idx, ws_id) in ws_ids.iter().enumerate() {
                if let Some(ws) = workspaces.get(idx) {
                    let is_active = ws.id == active;
                    
                    self.workspace_ids[idx] = Some(ws.id);
                    
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
                        label = {
                            draw_text: {
                                active: (if is_active { 1.0 } else { 0.0 })
                            }
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
