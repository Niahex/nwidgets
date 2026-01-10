use cef::{Browser, ImplBrowser, ImplFrame};
use once_cell::sync::OnceCell;

static BROWSER: OnceCell<Browser> = OnceCell::new();

pub struct CefBrowser;

impl CefBrowser {
    pub fn get() -> Option<&'static Browser> {
        BROWSER.get()
    }

    pub fn set(browser: Browser) {
        BROWSER.set(browser).ok();
    }

    pub fn navigate(url: &str) {
        if let Some(browser) = Self::get() {
            if let Some(frame) = browser.main_frame() {
                frame.load_url(Some(&cef::CefString::from(url)));
            }
        }
    }
}
