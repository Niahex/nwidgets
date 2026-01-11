use crate::services::cef::BrowserView;
use crate::services::chat::{ChatService, ChatToggled};
use gpui::prelude::*;
use gpui::{div, AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window};
use std::path::PathBuf;

const DEFAULT_URL: &str = "https://gemini.google.com/app";

// Minified CSS for better performance
const CSS: &str = ":root .dark-theme{--bard-color-synthetic--chat-window-surface:#434c5e;--bard-color-neutral-90:#3b4252;--bard-color-neutral-95:#2e3440;--bard-color-neutral-96:#242933;--bard-color-enterprise-greeting:#4c566a;--bard-color-custom-primary-container:#2e3440;--bard-color-surface-tint:#88c0d0;--bard-color-footer-background:#242933;--bard-color-draft-chip-background:#3b4252;--bard-color-response-container-flipped-background:#2e3440;--bard-color-sidenav-tile-background-aurora:#3b4252;--bard-color-sidenav-tile-text-aurora:#d8dee9;--bard-color-tertiary-60:#a3be8c;--bard-color-share-link:#88c0d0;--bard-color-chrome-experiment-badge:#81a1c1;--bard-color-onegooglebar-product-controls:rgba(236,239,244,0.87);--bard-color-code-comment:#616e88;--bard-color-code-variables:#d8dee9;--bard-color-code-literal:#d08770;--bard-color-code-class:#ebcb8b;--bard-color-code-string:#a3be8c;--bard-color-code-quotes-and-meta:#8fbcbb;--bard-color-code-keyword:#b48ead;--bard-color-code-chip-foreground:#eceff4;--bard-color-code-chip-background:#434c5e;--bard-color-code-chip-background-selected:#5e81ac;--bard-color-fact-check-tooltip-entailed-highlight:rgba(163,190,140,0.25);--bard-color-fact-check-tooltip-entailed-selected:#a3be8c;--bard-color-fact-check-tooltip-contradictory-highlight:rgba(191,97,106,0.25);--bard-color-fact-check-tooltip-contradictory-selected:#bf616a;--bard-color-fact-title:#a3be8c;--bard-color-fact-title-invalid:#d08770;--bard-color-recitation-background:rgba(136,192,208,0.2);--bard-color-bard-avatar-v2-basic-circle-stop-1:#5e81ac;--bard-color-bard-avatar-v2-basic-circle-stop-2:#81a1c1;--bard-color-bard-avatar-v2-basic-circle-stop-3:#88c0d0;--bard-color-bard-avatar-v2-advanced-circle-stop-1:#b48ead;--bard-color-bard-avatar-v2-advanced-circle-stop-2:#bf616a;--bard-color-bard-avatar-v2-advanced-circle-stop-3:#d08770;--bard-color-brand-text-gradient-stop-1:#81a1c1;--bard-color-brand-text-gradient-stop-2:#b48ead;--bard-color-brand-text-gradient-stop-3:#bf616a;--bard-color-form-field-outline:#4c566a;--bard-color-form-field-outline-active:#88c0d0;--bard-color-form-field-outline-hover:#d8dee9;--bard-color-input-area-buttons-hover-background:#434c5e;--bard-color-input-companion-button-hover-background:#434c5e;--bard-color-input-companion-button-active-background:#4c566a;--bard-color-planner-status-indicator-bar-error:#bf616a;--bard-color-planner-status-indicator-bar-warning:#ebcb8b;--bard-color-planner-status-indicator-bar-update:#81a1c1;--bard-color-lightbox-background:#242933;--bard-color-skeleton-loader-background-1:#2e3440;--bard-color-skeleton-loader-background-2:#3b4252;--bard-color-skeleton-loader-background-3:#434c5e;--bard-color-icon-separator:#4c566a}:where(.theme-host):where(.dark-theme){--gem-sys-color--brand-blue:#81a1c1;--gem-sys-color--brand-floaty-blue:#88c0d0;--gem-sys-color--brand-green:#a3be8c;--gem-sys-color--brand-red:#bf616a;--gem-sys-color--brand-transition-blue-1:#5e81ac;--gem-sys-color--brand-transition-blue-2:#81a1c1;--gem-sys-color--brand-transition-orange:#d08770;--gem-sys-color--brand-transition-pink:#b48ead;--gem-sys-color--brand-transition-purple:#b48ead;--gem-sys-color--brand-transition-red:#bf616a;--gem-sys-color--brand-transition-teal:#8fbcbb;--gem-sys-color--brand-yellow:#ebcb8b;--gem-sys-color--error:#bf616a;--gem-sys-color--on-error:#2e3440;--gem-sys-color--error-container:#4c566a;--gem-sys-color--surface:#2e3440;--gem-sys-color--surface-bright:#3b4252;--gem-sys-color--surface-container:#3b4252;--gem-sys-color--surface-container-high:#434c5e;--gem-sys-color--surface-container-highest:#4c566a;--gem-sys-color--surface-container-low:#2e3440;--gem-sys-color--surface-container-lowest:#242933;--gem-sys-color--surface-dim:#2e3440;--gem-sys-color--surface-variant:#434c5e;--gem-sys-color--on-surface:#eceff4;--gem-sys-color--on-surface-low:#d8dee9;--gem-sys-color--on-surface-variant:#e5e9f0;--gem-sys-color--inverse-surface:#eceff4;--gem-sys-color--inverse-on-surface:#2e3440;--gem-sys-color--primary:#88c0d0;--gem-sys-color--on-primary:#2e3440;--gem-sys-color--primary-container:#5e81ac;--gem-sys-color--on-primary-container:#eceff4;--gem-sys-color--primary-fixed:#81a1c1;--gem-sys-color--primary-fixed-dim:#5e81ac;--gem-sys-color--secondary:#8fbcbb;--gem-sys-color--on-secondary:#2e3440;--gem-sys-color--secondary-container:#4c566a;--gem-sys-color--on-secondary-container:#e5e9f0;--gem-sys-color--tertiary:#b48ead;--gem-sys-color--on-tertiary:#2e3440;--gem-sys-color--tertiary-container:#4c566a;--gem-sys-color--outline:#4c566a;--gem-sys-color--outline-low:#3b4252;--gem-sys-color--outline-variant:#434c5e;--gem-sys-color--blue-primary:#81a1c1;--gem-sys-color--blue-medium:#88c0d0;--gem-sys-color--blue-low:#5e81ac;--gem-sys-color--green-primary:#a3be8c;--gem-sys-color--green-low:#4c566a;--gem-sys-color--orange-primary:#d08770;--gem-sys-color--orange-low:#434c5e;--gem-sys-color--purple-primary:#b48ead;--gem-sys-color--purple-low:#4c566a;--gem-sys-color--red-primary:#bf616a;--gem-sys-color--red-low:#434c5e;--gem-sys-color--shadow:#242933;--gem-sys-color--scrim:#242933}:selection{background:rgba(143,188,187,0.5)}::-webkit-scrollbar{display:none}::target-text{background-color:#ebcb8b!important;color:#2e3440!important}::target-text:current{background-color:#d08770!important;color:#2e3440!important}";

pub struct ChatWidget {
    browser: Entity<BrowserView>,
    chat_service: Entity<ChatService>,
}

fn state_file() -> PathBuf {
    dirs::state_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("nwidgets")
        .join("chat_url")
}

fn load_url() -> String {
    std::fs::read_to_string(state_file()).unwrap_or_else(|_| DEFAULT_URL.to_string())
}

pub fn save_url(url: &str) {
    if let Some(parent) = state_file().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(state_file(), url);
}

impl ChatWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let url = load_url();

        // Pre-calculate the full injection script to avoid repeated string formatting and escaping
        let injection_script = format!(
            "const s=document.createElement('style');s.textContent=`{}`;document.head.appendChild(s);",
            CSS.replace('`', "\\`").replace("${ ", "\\${ ")
        );

        let browser = cx.new(|cx| BrowserView::new(&url, 600, 1370, Some(&injection_script), cx));
        let chat_service = ChatService::global(cx);

        browser.read(cx).set_hidden(true);

        let browser_clone = browser.clone();
        cx.subscribe(
            &chat_service,
            move |_this, service, _event: &ChatToggled, cx| {
                let visible = service.read(cx).visible;
                browser_clone.read(cx).set_hidden(!visible);
                cx.notify();
            },
        )
        .detach();

        Self {
            browser,
            chat_service,
        }
    }

    pub fn current_url(&self, cx: &gpui::App) -> Option<String> {
        self.browser.read(cx).current_url()
    }

    pub fn navigate(&self, url: &str, cx: &mut gpui::App) {
        self.browser.read(cx).navigate(url);
    }
}

impl gpui::Render for ChatWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.chat_service.read(cx).visible;

        if !visible {
            return gpui::Empty.into_any_element();
        }

        let theme = cx.global::<crate::theme::Theme>();

        div()
            .id("chat-root")
            .size_full()
            .occlude()
            .bg(theme.bg)
            .rounded(gpui::px(18.))
            .overflow_hidden()
            .border_1()
            .border_color(theme.accent_alt.opacity(0.25))
            .shadow_lg()
            .child(self.browser.clone())
            .into_any_element()
    }
}
