use crate::services::hyprland::Workspace;
use crate::theme::*;
use gpui::{div, prelude::*, rgb};

pub struct WorkspaceModule {
    workspaces: Vec<Workspace>,
    active_workspace: i32,
}

impl WorkspaceModule {
    pub fn new(workspaces: Vec<Workspace>, active_workspace: i32) -> Self {
        Self {
            workspaces,
            active_workspace,
        }
    }

    pub fn update(&mut self, workspaces: Vec<Workspace>, active_workspace: i32) {
        self.workspaces = workspaces;
        self.active_workspace = active_workspace;
    }

    pub fn render(&self) -> impl IntoElement {
        let mut sorted_workspaces = self.workspaces.clone();
        // Sort: 1-6 first, then others
        sorted_workspaces.sort_by(|a, b| match (a.id <= 6, b.id <= 6) {
            (true, true) => a.id.cmp(&b.id),
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            (false, false) => a.id.cmp(&b.id),
        });

        div().flex().flex_row().items_center().gap_2().children(
            sorted_workspaces
                .into_iter()
                .take(8)
                .map(|ws| {
                    let is_active = ws.id == self.active_workspace;
                    let color = if is_active {
                        colors::frost0(100)
                    } else {
                        colors::snow0(100)
                    };
                    let bg_color = if is_active {
                        colors::frost0(35)
                    } else {
                        colors::polar2(100)
                    };
                    div()
                        .w_8()
                        .h_8()
                        .bg(bg_color)
                        .rounded_sm()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(color)
                        .text_xs()
                        .child(ws.id.to_string())
                })
                .collect::<Vec<_>>(),
        )
    }
}
