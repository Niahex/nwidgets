use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;
    use crate::theme::*;

    pub NwButton = <Button> {
        width: Fit, height: Fit
        padding: {left: 12, right: 12, top: 8, bottom: 8}

        draw_bg: {
            color: (NORD_POLAR_2)
            radius: 6.0
        }
        draw_text: {
            color: (THEME_COLOR_TEXT_DEFAULT)
            text_style: (THEME_FONT_REGULAR) { font_size: 12.0 }
        }
    }

    pub NwButtonAccent = <NwButton> {
        draw_bg: {
            color: (NORD_FROST_1)
        }
        draw_text: {
            color: (NORD_POLAR_0)
            text_style: (THEME_FONT_BOLD) { font_size: 12.0 }
        }
    }

    pub NwButtonDanger = <NwButton> {
        draw_bg: {
            color: (NORD_AURORA_RED)
        }
        draw_text: {
            color: (NORD_SNOW_2)
            text_style: (THEME_FONT_BOLD) { font_size: 12.0 }
        }
    }

    pub NwButtonGhost = <NwButton> {
        draw_bg: {
            color: #0000
        }
        draw_text: {
            color: (THEME_COLOR_TEXT_DEFAULT)
        }
    }

    pub NwIconButton = <Button> {
        width: 32, height: 32
        padding: 0
        align: {x: 0.5, y: 0.5}

        draw_bg: {
            color: #0000
            radius: 6.0
        }
        draw_icon: {
            color: (THEME_COLOR_TEXT_DEFAULT)
            svg_file: dep("")
        }
    }
}
