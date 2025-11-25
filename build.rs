use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Automatically watch all SCSS files in the styles directory
    if let Ok(entries) = fs::read_dir("styles") {
        for entry in entries.flatten() {
            if let Some(path) = entry.path().to_str() {
                if path.ends_with(".scss") {
                    println!("cargo:rerun-if-changed={}", path);
                }
            }
        }
    }

    // Compile SCSS to CSS
    let scss_path = "styles/main.scss";
    let css =
        grass::from_path(scss_path, &grass::Options::default()).expect("Failed to compile SCSS");

    // Generate the style.rs file
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_style.rs");

    let content = format!(
        "use gtk4::{{self as gtk, gdk}};
\nconst CSS: &str = r#\"{}\"#;
\npub fn load_css() {{
    let provider = gtk::CssProvider::new();
    provider.load_from_data(CSS);

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect(\"Could not connect to a display.\"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_USER,
    );
}}
",
        css
    );

    fs::write(&dest_path, content).expect("Failed to write generated_style.rs");

    // Generate icon loader from SVG files
    generate_icon_loader();
}

fn generate_icon_loader() {
    println!("cargo:rerun-if-changed=assets");

    let mut icon_entries = Vec::new();

    if let Ok(entries) = fs::read_dir("assets") {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "svg" {
                    if let Some(file_name) = path.file_stem() {
                        let icon_name = file_name.to_string_lossy().to_string();
                        let file_path = path.file_name().unwrap().to_string_lossy().to_string();
                        icon_entries.push((icon_name, file_path));
                    }
                }
            }
        }
    }

    icon_entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut code = String::from("[\n");

    for (icon_name, file_path) in &icon_entries {
        code.push_str(&format!(
            "    (\"{}\", &include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/assets/{}\"))[..]),\n",
            icon_name,
            file_path
        ));
    }

    code.push_str("]");

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("generated_icons.rs");
    fs::write(&dest_path, code).expect("Failed to write generated_icons.rs");
}