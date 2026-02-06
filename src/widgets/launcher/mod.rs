pub mod applications;
pub mod calculator;
pub mod core;
pub mod fuzzy;
pub mod process;
pub mod search_input;
pub mod search_results;
pub mod state;
pub mod widget;
pub mod window;

use gpui::*;

pub use core::{LauncherCore, SearchResultType};
pub use search_input::SearchInput;
pub use search_results::{SearchResult, SearchResults};
pub use widget::{LauncherWidget, Backspace, Down, Launch, Quit, Up};

#[derive(Clone)]
pub struct LauncherToggled;

pub struct LauncherService {
    pub visible: bool,
}

impl EventEmitter<LauncherToggled> for LauncherService {}

impl LauncherService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self { visible: false }
    }

    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.visible = !self.visible;

        // Close control center when launcher opens
        if self.visible {
            let cc = crate::services::ui::control_center::ControlCenterService::global(cx);
            cc.update(cx, |cc, cx| {
                if cc.is_visible() {
                    cc.toggle(cx);
                }
            });
        }

        cx.emit(LauncherToggled);
        cx.notify();
    }
}

// Global accessor
struct GlobalLauncherService(Entity<LauncherService>);
impl Global for GlobalLauncherService {}

impl LauncherService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalLauncherService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalLauncherService(service.clone()));

        // Scan applications in background at startup
        cx.spawn(|cx: &mut AsyncApp| {
            let cx = cx.clone();
            async move {
                cx.background_executor()
                    .spawn(async {
                        let apps = applications::scan_applications();
                        let _ = applications::save_to_cache(&apps);
                    })
                    .await;
            }
        })
        .detach();

        service
    }
}
