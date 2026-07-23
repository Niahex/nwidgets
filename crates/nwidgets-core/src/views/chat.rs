use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::corner::{Corner, CornerPosition};
use gpui_component::input::{Input, InputState};
use gpui_component::Icon;

const CHAT_WIDTH: f32 = 600.0;
const CORNER_RADIUS: f32 = 12.0;
const MAIN_WIDTH: f32 = CHAT_WIDTH - CORNER_RADIUS; // 588.0px

actions!(chat, [CloseChat, SendMessage]);

#[derive(Clone)]
pub struct ChatMessage {
    pub role: String, // "user" or "assistant"
    pub content: String,
    pub thinking: Option<String>,
}

pub struct Chat {
    pub focus_handle: FocusHandle,
    pub messages: Vec<ChatMessage>,
    pub input_state: Entity<InputState>,
    pub thinking_expanded: bool,
    pub is_generating: bool,
}

impl Chat {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input_state = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_placeholder("Posez votre question à l'assistant IA...", window, cx);
            state
        });

        // Sample initial messages to demonstrate full UI
        let messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Comment puis-je vérifier l'état des services nwidgets ?".to_string(),
                thinking: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Vous pouvez vérifier l'état des services en utilisant le service de système `nwidgets-service-system-monitor` ou en lançant la commande `systemctl --user status nwidgets`.".to_string(),
                thinking: Some("Analyse de la demande utilisateur...\nIdentification des crates de services disponibles (system_monitor, niri, audio, network).\nFormatage de la réponse explicative en français.".to_string()),
            },
        ];

        Self {
            focus_handle: cx.focus_handle(),
            messages,
            input_state,
            thinking_expanded: true,
            is_generating: false,
        }
    }

    pub fn send_current_message(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input_state.read(cx).value().trim().to_string();
        if text.is_empty() {
            return;
        }

        // Clear input
        self.input_state.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });

        // Add User Message
        self.messages.push(ChatMessage {
            role: "user".to_string(),
            content: text.clone(),
            thinking: None,
        });

        // Add Assistant response simulation
        self.messages.push(ChatMessage {
            role: "assistant".to_string(),
            content: format!("J'ai bien reçu votre demande : \"{}\". Je traite les informations avec le système nwidgets.", text),
            thinking: Some("Traitement de la requête en cours via Kaji AI Router...\nAnalyse du contexte système.".to_string()),
        });

        cx.notify();
    }

    pub fn clear_messages(&mut self, cx: &mut Context<Self>) {
        self.messages.clear();
        cx.notify();
    }

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let text_main = rgb(0xe5e9f0);
        let text_muted = rgb(0x4c566a);
        let accent = rgb(0x88c0d0);
        let green = rgb(0xa3be8c);
        let frost_border = rgb(0x88c0d0).opacity(0.3);

        div()
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_3()
            .border_b_1()
            .border_color(frost_border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .p_2()
                            .bg(rgb(0x3b4252))
                            .rounded_lg()
                            .child(Icon::new("smart_toy").size(px(20.0)).text_color(accent)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(text_main)
                                    .child("Assistant IA"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1_5()
                                    .child(div().size(px(6.0)).rounded_full().bg(green))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(text_muted)
                                            .child("En ligne • Kaji Core"),
                                    ),
                            ),
                    ),
            )
            .child(
                div()
                    .id("clear-chat-btn")
                    .cursor_pointer()
                    .p_1_5()
                    .rounded_md()
                    .hover(|s| s.bg(rgb(0x3b4252)))
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.clear_messages(cx);
                    }))
                    .child(Icon::new("delete_sweep").size(px(18.0)).text_color(text_muted)),
            )
    }

    fn render_empty_state(&self) -> impl IntoElement {
        let text_main = rgb(0xe5e9f0);
        let text_muted = rgb(0x4c566a);
        let accent = rgb(0x88c0d0);
        let card_bg = rgb(0x3b4252);

        div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap_4()
            .p_6()
            .child(
                div()
                    .p_4()
                    .bg(card_bg)
                    .rounded_full()
                    .child(Icon::new("smart_toy").size(px(40.0)).text_color(accent)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .child(
                        div()
                            .text_base()
                            .font_weight(FontWeight::BOLD)
                            .text_color(text_main)
                            .child("Comment puis-je vous aider ?"),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(text_muted)
                            .child("Posez des questions sur le système, le code ou la configuration."),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w_full()
                    .max_w(px(320.0))
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .bg(card_bg)
                            .rounded_lg()
                            .text_xs()
                            .text_color(text_main)
                            .child("⚡ Activer le mode haute performance CPU"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_2()
                            .bg(card_bg)
                            .rounded_lg()
                            .text_xs()
                            .text_color(text_main)
                            .child("🔍 Analyser les journaux de nwidgets-core"),
                    ),
            )
    }

    fn render_message(&self, msg: &ChatMessage, ix: usize) -> impl IntoElement {
        let text_main = rgb(0xe5e9f0);
        let text_muted = rgb(0xd8dee9);
        let card_bg = rgb(0x3b4252);
        let purple = rgb(0xb48ead);
        let is_user = msg.role == "user";
        let thinking_expanded = self.thinking_expanded;

        div()
            .flex()
            .flex_col()
            .gap_1_5()
            .w_full()
            .when(is_user, |this| {
                this.child(
                    div()
                        .flex()
                        .justify_end()
                        .child(
                            div()
                                .max_w(px(340.0))
                                .p_3()
                                .bg(rgb(0x4c566a))
                                .rounded_xl()
                                .text_sm()
                                .text_color(text_main)
                                .child(msg.content.clone()),
                        ),
                )
            })
            .when(!is_user, |_this| {
                let mut container = div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .w_full()
                    .p_3()
                    .bg(card_bg)
                    .rounded_xl();

                // Thinking block
                if let Some(ref think_text) = msg.thinking {
                    let chevron_icon = if thinking_expanded { "expand_less" } else { "expand_more" };
                    container = container.child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .id(SharedString::from(format!("think-toggle-{}", ix)))
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(Icon::new("psychology").size(px(14.0)).text_color(purple))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(purple)
                                            .child("Raisonnement"),
                                    )
                                    .child(div().flex_1())
                                    .child(Icon::new(chevron_icon).size(px(14.0)).text_color(purple)),
                            )
                            .when(thinking_expanded, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .pl_2()
                                        .child(div().w(px(2.0)).bg(purple))
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(text_muted)
                                                .child(think_text.clone()),
                                        ),
                                )
                            }),
                    );
                }

                // Assistant Response Content
                container.child(
                    div()
                        .text_sm()
                        .text_color(text_main)
                        .child(msg.content.clone()),
                )
            })
    }

    fn render_input_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);
        let text_muted = rgb(0xd8dee9);
        let accent = rgb(0x88c0d0);
        let frost_border = rgb(0x88c0d0).opacity(0.3);

        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .bg(bg)
            .border_t_1()
            .border_color(frost_border)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                if event.keystroke.key == "enter" {
                    this.send_current_message(window, cx);
                }
            }))
            .child(
                div()
                    .h(px(40.0))
                    .child(Input::new(&self.input_state).size_full()),
            )
            .child(
                div()
                    .w_full()
                    .flex()
                    .items_center()
                    .justify_between()
                    .h(px(32.0))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                div()
                                    .cursor_pointer()
                                    .p_1()
                                    .rounded_md()
                                    .hover(|s| s.bg(rgb(0x4c566a)))
                                    .child(Icon::new("add").size(px(16.0)).text_color(text_muted)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_1()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(rgb(0x3b4252))
                                    .child(Icon::new("psychology").size(px(14.0)).text_color(accent))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(accent)
                                            .child("Thinking"),
                                    ),
                            ),
                    )
                    .child(
                        div()
                            .id("send-message-btn")
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(28.0))
                            .bg(accent)
                            .rounded_lg()
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.send_current_message(window, cx);
                            }))
                            .child(Icon::new("send").size(px(16.0)).text_color(rgb(0x2e3440))),
                    ),
            )
    }
}

impl Render for Chat {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let bg = rgb(0x2e3440);
        let frost_border = rgb(0x88c0d0).opacity(0.3);

        // Force focus sur la barre de recherche / saisie
        let input_fh = self.input_state.read(cx).focus_handle(cx);
        window.focus(&input_fh, cx);

        let is_empty = self.messages.is_empty();

        div()
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|_this, _action: &CloseChat, _window, cx| {
                cx.emit(CloseChat);
            }))
            .w(px(CHAT_WIDTH))
            .h_full()
            .flex()
            .flex_row()
            // ── Main Chat Container (588px) ──
            .child(
                div()
                    .w(px(MAIN_WIDTH))
                    .h_full()
                    .bg(bg)
                    .border_b_1()
                    .border_color(frost_border)
                    .flex()
                    .flex_col()
                    .child(self.render_header(cx))
                    .child(
                        div().flex_1().flex().flex_col().when(is_empty, |this| {
                            this.child(self.render_empty_state())
                        }).when(!is_empty, |this| {
                            let msgs: Vec<_> = self.messages.iter().enumerate().map(|(ix, msg)| {
                                self.render_message(msg, ix)
                            }).collect();
                            this.child(
                                div()
                                    .w_full()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .p_3()
                                    .children(msgs)
                            )
                        })
                    )
                    .child(self.render_input_bar(cx)),
            )
            // ── Right Concave Corners Column (12px) ──
            .child(
                div()
                    .w(px(CORNER_RADIUS))
                    .h_full()
                    .flex()
                    .flex_col()
                    .child(
                        // Top-Left concave corner (under top bar)
                        Corner::new(CornerPosition::TopLeft, px(CORNER_RADIUS))
                            .color(bg)
                            .border_color(frost_border),
                    )
                    .child(
                        // Vertical border line in the right column
                        div().flex_1().flex().justify_start().child(
                            div().w(px(1.0)).h_full().bg(frost_border),
                        ),
                    )
                    .child(
                        // Bottom-Left concave corner
                        Corner::new(CornerPosition::BottomLeft, px(CORNER_RADIUS))
                            .color(bg)
                            .border_color(frost_border),
                    ),
            )
    }
}

impl EventEmitter<CloseChat> for Chat {}
