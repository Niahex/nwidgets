use gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::{gdk, glib, Image};
use resvg::usvg;

// Re-introduce embedding the SVG data
static SINK_MUTED: &[u8] = include_bytes!("../assets/sink-muted.svg");
static SINK_ZERO: &[u8] = include_bytes!("../assets/sink-zero.svg");
static SINK_LOW: &[u8] = include_bytes!("../assets/sink-low.svg");
static SINK_MEDIUM: &[u8] = include_bytes!("../assets/sink-medium.svg");
static SINK_HIGH: &[u8] = include_bytes!("../assets/sink-high.svg");
static SOURCE_MUTED: &[u8] = include_bytes!("../assets/source-muted.svg");
static SOURCE_ZERO: &[u8] = include_bytes!("../assets/source-zero.svg");
static SOURCE_LOW: &[u8] = include_bytes!("../assets/source-low.svg");
static SOURCE_MEDIUM: &[u8] = include_bytes!("../assets/source-medium.svg");
static SOURCE_HIGH: &[u8] = include_bytes!("../assets/source-high.svg");
static BLUETOOTH_PAIRED: &[u8] = include_bytes!("../assets/bluetooth-paired.svg");
static BLUETOOTH_ACTIVE: &[u8] = include_bytes!("../assets/bluetooth-active.svg");
static BLUETOOTH_DISABLED: &[u8] = include_bytes!("../assets/bluetooth-disabled.svg");
static DIALOG_INFO: &[u8] = include_bytes!("../assets/info.svg");
static COPY: &[u8] = include_bytes!("../assets/copy.svg");
static TEST_SVG: &[u8] = include_bytes!("../assets/test.svg");
static FIREFOX: &[u8] = include_bytes!("../assets/firefox.svg");
static DISCORD: &[u8] = include_bytes!("../assets/discord.svg");
static STEAM: &[u8] = include_bytes!("../assets/steam.svg");
static STEAM_TRAY: &[u8] = include_bytes!("../assets/steam_tray.svg");
static TERMINAL: &[u8] = include_bytes!("../assets/terminal.svg");
static ZEDITOR: &[u8] = include_bytes!("../assets/zeditor.svg");
static CAPSLOCK_ON: &[u8] = include_bytes!("../assets/capslock-on.svg");
static CAPSLOCK_OFF: &[u8] = include_bytes!("../assets/capslock-off.svg");
static FILE_MANAGER: &[u8] = include_bytes!("../assets/file-manager.svg");
static PIN: &[u8] = include_bytes!("../assets/pin.svg");
static UNPIN: &[u8] = include_bytes!("../assets/unpin.svg");
static PLAY: &[u8] = include_bytes!("../assets/play.svg");
static PAUSE: &[u8] = include_bytes!("../assets/pause.svg");
static COFFEE: &[u8] = include_bytes!("../assets/coffee.svg");
static CLIPBOARD: &[u8] = include_bytes!("../assets/clipboard.svg");

fn svg_to_pixbuf(data: &[u8], target_size: Option<u32>) -> Option<Pixbuf> {
    let opt = usvg::Options::default();
    let tree = usvg::Tree::from_data(data, &opt).ok()?;

    let size = tree.size();
    let (width, height) = if let Some(target) = target_size {
        let aspect = size.width() / size.height();
        if aspect > 1.0 {
            (target, (target as f32 / aspect) as u32)
        } else {
            ((target as f32 * aspect) as u32, target)
        }
    } else {
        (size.width() as u32, size.height() as u32)
    };

    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height)?;

    let transform = resvg::tiny_skia::Transform::from_scale(
        width as f32 / size.width(),
        height as f32 / size.height(),
    );

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    let bytes = glib::Bytes::from(pixmap.data());
    Some(Pixbuf::from_bytes(
        &bytes,
        Colorspace::Rgb,
        true, // has_alpha
        8,    // bits_per_sample
        width as i32,
        height as i32,
        (width * 4) as i32, // row_stride
    ))
}

pub fn setup_icon_theme() {
    // Rien à faire
}

pub fn get_paintable(icon_name: &str) -> Option<gdk::Texture> {
    get_paintable_with_size(icon_name, None)
}

pub fn get_paintable_with_size(icon_name: &str, size: Option<u32>) -> Option<gdk::Texture> {
    let svg_data = match icon_name {
        "sink-muted" => SINK_MUTED,
        "sink-zero" => SINK_ZERO,
        "sink-low" => SINK_LOW,
        "sink-medium" => SINK_MEDIUM,
        "sink-high" => SINK_HIGH,
        "source-muted" => SOURCE_MUTED,
        "source-zero" => SOURCE_ZERO,
        "source-low" => SOURCE_LOW,
        "source-medium" => SOURCE_MEDIUM,
        "source-high" => SOURCE_HIGH,
        "bluetooth-paired" => BLUETOOTH_PAIRED,
        "bluetooth-active" => BLUETOOTH_ACTIVE,
        "bluetooth-disabled" => BLUETOOTH_DISABLED,
        "dialog-information" => DIALOG_INFO,
        "copy" => COPY,
        "test" => TEST_SVG,
        "firefox" => FIREFOX,
        "discord" => DISCORD,
        "steam" => STEAM,
        "steam-tray" => STEAM_TRAY,
        "terminal" => TERMINAL,
        "zeditor" => ZEDITOR,
        "capslock-on" => CAPSLOCK_ON,
        "capslock-off" => CAPSLOCK_OFF,
        "file-manager" => FILE_MANAGER,
        "pin" => PIN,
        "unpin" => UNPIN,
        "play" => PLAY,
        "pause" => PAUSE,
        "coffee" => COFFEE,
        "clipboard" => CLIPBOARD,

        _ => {
            println!("DEBUG: Icon not found for name: {}", icon_name);
            return None;
        }
    };

    match svg_to_pixbuf(svg_data, size) {
        Some(pixbuf) => Some(gdk::Texture::for_pixbuf(&pixbuf)),
        None => {
            println!("DEBUG: Failed to create pixbuf for icon '{}'", icon_name);
            None
        }
    }
}

pub fn create_icon(icon_name: &str) -> Image {
    create_icon_with_size(icon_name, None)
}

pub fn create_icon_with_size(icon_name: &str, size: Option<u32>) -> Image {
    if let Some(texture) = get_paintable_with_size(icon_name, size) {
        Image::from_paintable(Some(&texture))
    } else {
        Image::new()
    }
}
