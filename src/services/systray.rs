use gpui::prelude::*;
use gpui::{App, Context, Entity, EventEmitter, Global};
use parking_lot::RwLock;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct SystrayItem {
    pub id: String,
    pub title: String,
    pub icon_name: Option<String>,
}

#[derive(Clone)]
pub struct SystrayChanged {
    pub items: Vec<SystrayItem>,
}

pub struct SystrayService {
    items: Arc<RwLock<Vec<SystrayItem>>>,
}

impl EventEmitter<SystrayChanged> for SystrayService {}

impl SystrayService {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        // For now, just a stub. Real implementation would use StatusNotifierWatcher DBus
        Self {
            items: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn items(&self) -> Vec<SystrayItem> {
        self.items.read().clone()
    }

    pub fn add_item(&self, item: SystrayItem, cx: &mut Context<Self>) {
        self.items.write().push(item.clone());
        cx.emit(SystrayChanged {
            items: self.items(),
        });
        cx.notify();
    }

    pub fn remove_item(&self, id: &str, cx: &mut Context<Self>) {
        self.items.write().retain(|item| item.id != id);
        cx.emit(SystrayChanged {
            items: self.items(),
        });
        cx.notify();
    }
}

// Global accessor
struct GlobalSystrayService(Entity<SystrayService>);
impl Global for GlobalSystrayService {}

impl SystrayService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalSystrayService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalSystrayService(service.clone()));
        service
    }
}
