use crate::services::cef::BrowserView;
use crate::services::discord::{DiscordService, DiscordToggled};
use gpui::prelude::*;
use gpui::{div, AppContext, Context, Entity, IntoElement, ParentElement, Styled, Window};

const DISCORD_URL: &str = "https://discord.com/app";

// Nordic theme CSS + collapsible sidebar with toggle button
const CSS: &str = r#":root{--nord-dark1:#2e3440;--nord-dark2:#3b4252;--nord-dark3:#434c5e;--nord-dark4:#4c566a;--nord-light1:#d8dee9;--nord-light2:#e5e9f0;--nord-light3:#eceff4;--nord-green-blue:#8fbcbb;--nord-turquoise:#88c0d0;--nord-cyan:#81a1c1;--nord-blue:#5e81ac;--nord-red:#bf616a;--nord-orange:#d08770;--nord-yellow:#ebcb8b;--nord-green:#a3be8c;--nord-pink:#b48ead}.theme-dark{--background-primary:var(--nord-dark1);--background-secondary:var(--nord-dark2);--background-secondary-alt:var(--nord-dark2);--background-tertiary:var(--nord-dark3);--background-accent:var(--nord-dark4);--background-floating:var(--nord-dark2);--background-modifier-hover:rgba(79,84,92,0.4);--background-modifier-active:rgba(79,84,92,0.6);--background-modifier-selected:rgba(79,84,92,0.8);--channeltextarea-background:var(--nord-dark3);--text-normal:var(--nord-light1);--text-muted:var(--nord-dark4);--text-link:var(--nord-turquoise);--interactive-normal:var(--nord-light1);--interactive-hover:var(--nord-light2);--interactive-active:var(--nord-light3);--interactive-muted:var(--nord-dark4);--header-primary:var(--nord-light2);--header-secondary:var(--nord-light1);--brand-experiment:var(--nord-green-blue);--brand-experiment-560:var(--nord-turquoise)}::selection{background-color:rgba(136,192,208,0.25)}[class*="guilds"]{width:0!important;overflow:hidden;transition:width .2s}body.sidebar-open [class*="guilds"]{width:72px!important}[class*="sidebarList"],[class*="sidebarListRounded"]{width:0!important;overflow:hidden;position:absolute;z-index:10;left:0;top:0;bottom:0;transition:width .2s}body.sidebar-open [class*="sidebarList"],body.sidebar-open [class*="sidebarListRounded"]{width:240px!important}[class*="membersWrap"]{width:0!important;min-width:0!important;overflow:hidden;transition:width .2s}[class*="membersWrap"]:hover{width:240px!important}#sidebar-toggle{position:fixed;left:8px;top:50%;transform:translateY(-50%);z-index:9999;width:24px;height:48px;background:var(--nord-dark3);border:none;border-radius:0 6px 6px 0;cursor:pointer;color:var(--nord-light1);font-size:14px;opacity:0.7;transition:opacity .2s}#sidebar-toggle:hover{opacity:1}"#;

const TOGGLE_JS: &str = r#"
if(!document.getElementById('sidebar-toggle')){
  const btn=document.createElement('button');
  btn.id='sidebar-toggle';
  btn.textContent='☰';
  btn.onclick=()=>document.body.classList.toggle('sidebar-open');
  document.body.appendChild(btn);
}"#;

pub struct DiscordWidget {
    browser: Option<Entity<BrowserView>>,
    discord_service: Entity<DiscordService>,
}

impl DiscordWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let discord_service = DiscordService::global(cx);

        cx.subscribe(
            &discord_service,
            |this, service, _event: &DiscordToggled, cx| {
                let visible = service.read(cx).visible;
                if visible && this.browser.is_none() {
                    this.browser = Some(cx.new(|cx| BrowserView::new(
                        DISCORD_URL, 1500, 1370, None, cx
                    )));
                }
                if let Some(browser) = &this.browser {
                    browser.read(cx).set_hidden(!visible);
                }
                cx.notify();
            },
        )
        .detach();

        Self {
            browser: None,
            discord_service,
        }
    }
}

impl gpui::Render for DiscordWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible = self.discord_service.read(cx).visible;

        if !visible || self.browser.is_none() {
            return gpui::Empty.into_any_element();
        }

        let theme = cx.global::<crate::theme::Theme>();

        div()
            .id("discord-root")
            .size_full()
            .occlude()
            .bg(theme.bg)
            .rounded(gpui::px(18.))
            .overflow_hidden()
            .border_1()
            .border_color(theme.accent_alt.opacity(0.25))
            .shadow_lg()
            .child(self.browser.clone().unwrap())
            .into_any_element()
    }
}
