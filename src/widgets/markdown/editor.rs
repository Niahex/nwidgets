use gtk4::{self as gtk, prelude::*};
use gtk4::{ScrolledWindow, CssProvider};
use gtk4::gdk::Display;
use std::rc::Rc;

use super::view::DocumentView;

/// Main markdown editor widget
pub struct MarkdownEditor {
    pub container: ScrolledWindow,
    document_view: Rc<DocumentView>,
}

impl MarkdownEditor {
    pub fn new() -> Self {
        Self::load_css();

        let document_view = Rc::new(DocumentView::new());

        let scrolled = ScrolledWindow::builder()
            .child(&document_view.container)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .vscrollbar_policy(gtk::PolicyType::Automatic)
            .build();

        scrolled.add_css_class("markdown-editor");
        document_view.container.add_css_class("markdown-document");

        Self {
            container: scrolled,
            document_view,
        }
    }

    fn load_css() {
        let css = r#"
.markdown-editor {
    background-color: #2e3440;
}

.markdown-document {
    background-color: #2e3440;
    color: #d8dee9;
    padding: 16px;
}

.markdown-divider {
    background-color: #4c566a;
    min-height: 2px;
}

textview.paragraph {
    font-size: 14px;
    color: #d8dee9;
}

textview.heading {
    font-weight: bold;
}

textview.heading-1 {
    font-size: 28px;
    color: #88c0d0;
}

textview.heading-2 {
    font-size: 24px;
    color: #81a1c1;
}

textview.heading-3 {
    font-size: 20px;
    color: #5e81ac;
}

textview.heading-4 {
    font-size: 18px;
    color: #5e81ac;
}

textview.heading-5 {
    font-size: 16px;
    color: #5e81ac;
}

textview.heading-6 {
    font-size: 14px;
    color: #5e81ac;
    font-weight: 600;
}

textview.list {
    color: #d8dee9;
}

textview.quote {
    color: #d8dee9;
    font-style: italic;
    border-left: 3px solid #5e81ac;
    padding-left: 12px;
}

textview.code {
    font-family: "JetBrains Mono", "Fira Code", "Courier New", monospace;
    font-size: 13px;
    background-color: #3b4252;
    color: #a3be8c;
    padding: 8px;
}

textview text {
    background-color: transparent;
}
"#;

        let provider = CssProvider::new();
        provider.load_from_data(css);

        if let Some(display) = Display::default() {
            gtk::style_context_add_provider_for_display(
                &display,
                &provider,
                gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }

    pub fn set_markdown(&self, text: &str) {
        self.document_view.load_markdown(text);
    }

    pub fn get_markdown(&self) -> String {
        self.document_view.export_markdown()
    }

    pub fn get_document_view(&self) -> &Rc<DocumentView> {
        &self.document_view
    }
}
