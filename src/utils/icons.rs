use gdk_pixbuf::{Colorspace, Pixbuf};
use gtk4::{gdk, glib, Image};
use once_cell::sync::Lazy;
use resvg::usvg;
use std::collections::HashMap;
use std::sync::Mutex;

static ICON_DATA: Lazy<HashMap<&'static str, &'static [u8]>> = Lazy::new(|| {
    let icons: &[(&str, &[u8])] = &include!(concat!(env!("OUT_DIR"), "/generated_icons.rs"));
    icons.iter().copied().collect()
});

static TEXTURE_CACHE: Lazy<Mutex<HashMap<(String, Option<u32>), gdk::Texture>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
    let key = (icon_name.to_string(), size);

    // Check cache first
    if let Ok(cache) = TEXTURE_CACHE.lock() {
        if let Some(texture) = cache.get(&key) {
            return Some(texture.clone());
        }
    }

    let svg_data = ICON_DATA.get(icon_name)?;

    match svg_to_pixbuf(svg_data, size) {
        Some(pixbuf) => {
            let texture = gdk::Texture::for_pixbuf(&pixbuf);
            // Store in cache
            if let Ok(mut cache) = TEXTURE_CACHE.lock() {
                cache.insert(key, texture.clone());
            }
            Some(texture)
        }
        None => {
            println!("DEBUG: Failed to create pixbuf for icon '{icon_name}'");
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