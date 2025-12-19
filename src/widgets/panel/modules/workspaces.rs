use crate::services::hyprland::{HyprlandService, WorkspaceChanged};
use gpui::prelude::*;
use gpui::*;

pub struct WorkspacesModule {
    hyprland: Entity<HyprlandService>,
}

impl WorkspacesModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let hyprland = HyprlandService::global(cx);

        // Subscribe to workspace changes
        cx.subscribe(
            &hyprland,
            |_this, _hyprland, _event: &WorkspaceChanged, cx| {
                cx.notify(); // Trigger re-render
            },
        )
        .detach();

        Self { hyprland }
    }
}

impl Render for WorkspacesModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut workspaces = self.hyprland.read(cx).workspaces();
        let active_id = self.hyprland.read(cx).active_workspace_id();
        let hyprland = self.hyprland.clone();

        // Sort workspaces in ascending order by ID
        workspaces.sort_by_key(|ws| ws.id);

        // Nord colors
        let frost1 = rgb(0x88c0d0);
        let polar1 = rgb(0x3b4252);
        let snow2 = rgb(0xeceff4);

        div()
            .flex()
            .gap_1()
            .children(workspaces.into_iter().map(|ws| {
                let is_active = ws.id == active_id;
                let ws_id = ws.id;
                let hyprland = hyprland.clone();

                // Format workspace name: if it's not a number, use first letter capitalized
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

                div()
                    .id(("workspace", ws.id as u32))
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .text_sm()
                    .font_weight(if is_active {
                        FontWeight::BOLD
                    } else {
                        FontWeight::MEDIUM
                    })
                    .when(is_active, |this| {
                        this.bg(rgba(0x88c0d033)) // frost1 with opacity
                            .text_color(frost1)
                    })
                    .when(!is_active, |this| {
                        this.text_color(rgba(0xd8dee980)) // snow0 with opacity
                            .hover(|style| {
                                style
                                    .bg(rgba(0x88c0d01a)) // frost1 hover
                                    .text_color(rgba(0xd8dee9cc))
                            })
                    })
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        hyprland.read(cx).switch_to_workspace(ws_id);
                    })
                    .child(display_name)
            }))
    }
}
