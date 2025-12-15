use gpui::prelude::*;
use gpui::*;
use crate::services::hyprland::{HyprlandService, WorkspaceChanged};

pub struct WorkspacesModule {
    hyprland: Entity<HyprlandService>,
}

impl WorkspacesModule {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let hyprland = HyprlandService::global(cx);

        // Subscribe to workspace changes
        cx.subscribe(&hyprland, |_this, _hyprland, _event: &WorkspaceChanged, cx| {
            cx.notify(); // Trigger re-render
        })
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

                div()
                    .id(("workspace", ws.id as u32))
                    .px_3()
                    .py_1()
                    .rounded_md()
                    .when(is_active, |this| {
                        this.bg(rgb(0x89b4fa))
                            .text_color(rgb(0x1e1e2e))
                    })
                    .when(!is_active, |this| {
                        this.bg(rgb(0x313244))
                            .text_color(rgb(0xcdd6f4))
                            .hover(|style| style.bg(rgb(0x45475a)))
                    })
                    .cursor_pointer()
                    .on_click(move |_event, _window, cx| {
                        hyprland.read(cx).switch_to_workspace(ws_id);
                    })
                    .child(ws.name.clone())
            }))
    }
}
