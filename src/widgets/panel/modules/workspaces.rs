use crate::services::system::hyprland::{HyprlandService, WorkspaceChanged};
use crate::theme::ActiveTheme;
use crate::components::Button;
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
        let workspaces = self.hyprland.read(cx).workspaces();
        let active_id = self.hyprland.read(cx).active_workspace_id();
        let hyprland = self.hyprland.clone();

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

                Button::new(("workspace", ws.id as u32))
                    .label(display_name)
                    .accent()
                    .selected(is_active)
                    .on_click(move |_event, _window, cx| {
                        hyprland.read(cx).switch_to_workspace(ws_id);
                    })
            }))
    }
}
