use cef::{Browser, BrowserSettings, CefString, ImplBrowser, ImplFrame, WindowInfo};
use cef::browser_host_create_browser;
use once_cell::sync::OnceCell;
use super::client::CefClient;

static BROWSER: OnceCell<Browser> = OnceCell::new();

pub struct CefBrowser;

impl CefBrowser {
    pub fn get() -> Option<&'static Browser> {
        BROWSER.get()
    }

    pub fn set(browser: Browser) {
        BROWSER.set(browser).ok();
    }

    pub fn create(url: &str, _width: i32, _height: i32) {
        let window_info = WindowInfo {
            windowless_rendering_enabled: 1,
            ..Default::default()
        };

        let mut client = CefClient::new();
        let url_cef = CefString::from(url);
        let settings = BrowserSettings::default();

        browser_host_create_browser(
            Some(&window_info),
            Some(&mut client),
            Some(&url_cef),
            Some(&settings),
            None,
            None,
        );
    }

    pub fn navigate(url: &str) {
        if let Some(browser) = Self::get() {
            if let Some(frame) = browser.main_frame() {
                frame.load_url(Some(&CefString::from(url)));
            }
        }
    }
}
