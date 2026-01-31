use makepad_widgets::*;

live_design! {
    use link::theme::*;
    use link::widgets::*;

    pub NORD_POLAR_0 = #2E3440
    pub NORD_POLAR_1 = #3B4252
    pub NORD_POLAR_2 = #434C5E
    pub NORD_POLAR_3 = #4C566A

    pub NORD_SNOW_0 = #D8DEE9
    pub NORD_SNOW_1 = #E5E9F0
    pub NORD_SNOW_2 = #ECEFF4

    pub NORD_FROST_0 = #8FBCBB
    pub NORD_FROST_1 = #88C0D0
    pub NORD_FROST_2 = #81A1C1
    pub NORD_FROST_3 = #5E81AC

    pub NORD_AURORA_RED = #BF616A
    pub NORD_AURORA_ORANGE = #D08770
    pub NORD_AURORA_YELLOW = #EBCB8B
    pub NORD_AURORA_GREEN = #A3BE8C
    pub NORD_AURORA_PURPLE = #B48EAD

    pub COLOR_MUTE = #3B425266
    pub COLOR_ACCENT = #88C0D040

    pub THEME_COLOR_BG_APP = (NORD_POLAR_0)
    pub THEME_COLOR_BG_CONTAINER = (NORD_POLAR_1)
    pub THEME_COLOR_BG_SURFACE = (NORD_POLAR_2)
    pub THEME_COLOR_BG_HOVER = (NORD_POLAR_3)

    pub THEME_COLOR_TEXT_DEFAULT = (NORD_SNOW_2)
    pub THEME_COLOR_TEXT_MUTE = (NORD_SNOW_0)
    pub THEME_COLOR_TEXT_BRIGHT = (NORD_SNOW_1)

    pub THEME_COLOR_ACCENT = (NORD_FROST_1)
    pub THEME_COLOR_ACCENT_ALT = (NORD_FROST_0)

    pub THEME_COLOR_RED = (NORD_AURORA_RED)
    pub THEME_COLOR_ORANGE = (NORD_AURORA_ORANGE)
    pub THEME_COLOR_YELLOW = (NORD_AURORA_YELLOW)
    pub THEME_COLOR_GREEN = (NORD_AURORA_GREEN)
    pub THEME_COLOR_PURPLE = (NORD_AURORA_PURPLE)

    pub THEME_FONT_REGULAR = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuNerdFont-Regular.ttf", 0.0, 0.0),
        }
    }

    pub THEME_FONT_BOLD = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuNerdFont-Bold.ttf", 0.0, 0.0),
        }
    }

    pub THEME_FONT_ITALIC = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuNerdFont-Italic.ttf", 0.0, 0.0),
        }
    }

    pub THEME_FONT_CODE = {
        font_family: {
            latin = font("crate://self/assets/fonts/UbuntuSansMonoNerdFont-Regular.ttf", 0.0, 0.0),
        }
    }

    pub NordLabel = <Label> {
        draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
    }

    pub NordLabelMuted = <Label> {
        draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 11.0 }, color: (THEME_COLOR_TEXT_MUTE) }
    }

    pub NordLabelBold = <Label> {
        draw_text: { text_style: <THEME_FONT_BOLD> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
    }

    pub NordButton = <Button> {
        draw_bg: { color: (NORD_POLAR_2) }
        draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
    }

    pub NordButtonAccent = <Button> {
        draw_bg: { color: (NORD_FROST_1) }
        draw_text: { text_style: <THEME_FONT_BOLD> { font_size: 12.0 }, color: (NORD_POLAR_0) }
    }

    pub NordButtonDanger = <Button> {
        draw_bg: { color: (NORD_AURORA_RED) }
        draw_text: { text_style: <THEME_FONT_BOLD> { font_size: 12.0 }, color: (NORD_SNOW_2) }
    }

    pub NordButtonGhost = <Button> {
        draw_bg: { color: #0000 }
        draw_text: { text_style: <THEME_FONT_REGULAR> { font_size: 12.0 }, color: (THEME_COLOR_TEXT_DEFAULT) }
    }

    pub NordView = <View> {
        show_bg: true
        draw_bg: { color: (THEME_COLOR_BG_APP) }
    }

    pub NordSurfaceView = <View> {
        show_bg: true
        draw_bg: { color: (THEME_COLOR_BG_CONTAINER) }
    }
}
