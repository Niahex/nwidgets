use super::app::CefApp;
use super::browser::CefBrowser;
use cef::{args::Args, execute_process, initialize, Settings, LogSeverity};
use gpui::{App, AppContext, Context, Entity, EventEmitter, Global};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone)]
pub struct CefReady;

#[derive(Clone)]
pub struct CefNavigated {
    pub url: String,
}

pub struct CefService {
    state: Arc<RwLock<CefState>>,
}

struct CefState {
    ready: bool,
    current_url: Option<String>,
    initialized: bool,
}

impl EventEmitter<CefReady> for CefService {}
impl EventEmitter<CefNavigated> for CefService {}

struct GlobalCefService(Entity<CefService>);
impl Global for GlobalCefService {}

impl CefService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalCefService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(|_cx| Self {
            state: Arc::new(RwLock::new(CefState {
                ready: false,
                current_url: None,
                initialized: false,
            })),
        });
        cx.set_global(GlobalCefService(service.clone()));
        service
    }

    pub fn is_ready(&self) -> bool {
        self.state.read().ready
    }

    pub fn current_url(&self) -> Option<String> {
        self.state.read().current_url.clone()
    }

    pub fn initialize(&mut self, cx: &mut Context<Self>) {
        let mut state = self.state.write();
        if state.initialized {
            return;
        }
        state.initialized = true;
        state.ready = true;
        drop(state);

        cx.emit(CefReady);
    }

    pub fn navigate(&mut self, url: String, cx: &mut Context<Self>) {
        self.state.write().current_url = Some(url.clone());
        
        // Create browser if it doesn't exist
        if CefBrowser::get().is_none() {
            CefBrowser::create(&url, 600, 1440);
        } else {
            CefBrowser::navigate(&url);
        }
        
        cx.emit(CefNavigated { url });
    }
}
