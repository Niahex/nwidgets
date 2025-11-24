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
        "use gtk4::{{self as gtk, gdk}};\n\
\n\
const CSS: &str = r#\"{}\"#;\n\
\n\
pub fn load_css() {{\n\
    let provider = gtk::CssProvider::new();\n\
    provider.load_from_data(CSS);\n\
\n\
    gtk::style_context_add_provider_for_display(\n\
        &gdk::Display::default().expect(\"Could not connect to a display.\"),\n\
        &provider,\n\
        gtk::STYLE_PROVIDER_PRIORITY_USER,\n\
    );\n\
}}\n",
        css
    );

    fs::write(&dest_path, content).expect("Failed to write generated_style.rs");
}
