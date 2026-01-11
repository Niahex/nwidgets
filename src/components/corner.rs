use gpui::*;

/// Corner position for the rounded corner widget
#[derive(Clone, Copy)]
pub enum CornerPosition {
    TopLeft,
    TopRight,
}

/// A widget that draws a rounded corner
pub struct Corner {
    position: CornerPosition,
    radius: Pixels,
}

impl Corner {
    pub fn new(position: CornerPosition, radius: Pixels) -> Self {
        Self { position, radius }
    }
}

impl Render for Corner {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<crate::theme::Theme>();
        let color = theme.bg;
        let r = self.radius;
        let position = self.position;

        canvas(
            move |_, _, _| {},
            move |bounds, _, window, _| {
                let ox = bounds.origin.x;
                let oy = bounds.origin.y;
                let mut path = PathBuilder::fill();

                match position {
                    CornerPosition::TopLeft => {
                        path.move_to(point(ox, oy + r));
                        path.arc_to(point(r, r), px(0.), false, true, point(ox + r, oy));
                        path.line_to(point(ox, oy));
                        path.close();
                    }
                    CornerPosition::TopRight => {
                        path.move_to(point(ox, oy));
                        path.arc_to(point(r, r), px(0.), false, true, point(ox + r, oy + r));
                        path.line_to(point(ox + r, oy));
                        path.close();
                    }
                }

                if let Ok(built_path) = path.build() {
                    window.paint_path(built_path, color);
                }
            },
        )
        .size(r)
    }
}
