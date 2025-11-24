use gdk_pixbuf::Pixbuf;
use gtk4::prelude::*;
use gtk4::{gdk, glib, Image};

// Embarquer les SVG directement dans le binaire
static SINK_MUTED: &[u8] = include_bytes!("../assets/status/sink-muted.svg");
static SINK_ZERO: &[u8] = include_bytes!("../assets/status/sink-zero.svg");
static SINK_LOW: &[u8] = include_bytes!("../assets/status/sink-low.svg");
static SINK_MEDIUM: &[u8] = include_bytes!("../assets/status/sink-medium.svg");
static SINK_HIGH: &[u8] = include_bytes!("../assets/status/sink-high.svg");

static SOURCE_MUTED: &[u8] = include_bytes!("../assets/status/source-muted.svg");
static SOURCE_ZERO: &[u8] = include_bytes!("../assets/status/source-zero.svg");
static SOURCE_LOW: &[u8] = include_bytes!("../assets/status/source-low.svg");
static SOURCE_MEDIUM: &[u8] = include_bytes!("../assets/status/source-medium.svg");
static SOURCE_HIGH: &[u8] = include_bytes!("../assets/status/source-high.svg");

static MEDIA_PLAY: &[u8] = include_bytes!("../assets/actions/24/media-playback-start.svg");
static MEDIA_PAUSE: &[u8] = include_bytes!("../assets/actions/24/media-playback-pause.svg");

static BLUETOOTH_PAIRED: &[u8] = include_bytes!("../assets/status/bluetooth-paired.svg");
static BLUETOOTH_ACTIVE: &[u8] = include_bytes!("../assets/status/bluetooth-active.svg");
static BLUETOOTH_DISABLED: &[u8] = include_bytes!("../assets/status/bluetooth-disabled.svg");

static DIALOG_INFO: &[u8] = include_bytes!("../assets/status/dialog-information.svg");
static COPY: &[u8] = include_bytes!("../assets/actions/copy.svg");

pub fn setup_icon_theme() {
    // Rien à faire
}

pub fn create_icon(icon_name: &str) -> Image {
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
        "media-playback-start" => MEDIA_PLAY,
        "media-playback-pause" => MEDIA_PAUSE,
        "bluetooth-paired" => BLUETOOTH_PAIRED,
        "bluetooth-active" => BLUETOOTH_ACTIVE,
        "bluetooth-disabled" => BLUETOOTH_DISABLED,
        "dialog-information" => DIALOG_INFO,
        "copy" => COPY,
        _ => return Image::new(), // Icône vide si non trouvée
    };

    let bytes = glib::Bytes::from_static(svg_data);
    if let Ok(pixbuf) = Pixbuf::from_stream(
        &gtk4::gio::MemoryInputStream::from_bytes(&bytes),
        None::<&gtk4::gio::Cancellable>,
    ) {
        let texture = gdk::Texture::for_pixbuf(&pixbuf);
        Image::from_paintable(Some(&texture))
    } else {
        Image::new()
    }
}
