use gpui::prelude::*;
use gpui::{
    div, Context, Entity, IntoElement, MouseButton, ParentElement, Render, SharedString, Styled, WeakEntity,
    Window,
};

use crate::theme::ActiveTheme;
use crate::widgets::dofustools::crafter::{CrafterService, CrafterStateChanged, ProfitabilityResult};

pub struct CrafterWidget {
    crafter_service: Entity<CrafterService>,
    search_query: SharedString,
    search_results: Vec<SearchResultItem>,
    selected_item: Option<SelectedItemData>,
    calculating: bool,
}

#[derive(Clone)]
struct SearchResultItem {
    id: i32,
    name: String,
}

#[derive(Clone)]
struct SelectedItemData {
    item_id: i32,
    item_name: String,
    result: Option<ProfitabilityResult>,
}

impl CrafterWidget {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let crafter_service = CrafterService::global(cx);

        cx.subscribe(&crafter_service, |this, _service, _event: &CrafterStateChanged, cx| {
            cx.notify();
        })
        .detach();

        Self {
            crafter_service,
            search_query: SharedString::from(""),
            search_results: Vec::new(),
            selected_item: None,
            calculating: false,
        }
    }

    fn search_items(&mut self, query: String, cx: &mut Context<Self>) {
        if query.len() < 2 {
            self.search_results.clear();
            cx.notify();
            return;
        }

        let crafter_service = self.crafter_service.clone();
        let this = cx.entity().downgrade();
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Ok(items) = crafter_service.read(&cx).search_items(query).await {
                let results: Vec<SearchResultItem> = items
                    .into_iter()
                    .map(|item| SearchResultItem {
                        id: item.id,
                        name: item
                            .name
                            .get("fr")
                            .or_else(|| item.name.get("en"))
                            .cloned()
                            .unwrap_or_else(|| format!("Item {}", item.id)),
                    })
                    .collect();
                let _ = this.update(cx, |this: &mut Self, cx: &mut Context<Self>| {
                    this.search_results = results;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn select_item(&mut self, item_id: i32, item_name: String, cx: &mut Context<Self>) {
        self.selected_item = Some(SelectedItemData {
            item_id,
            item_name,
            result: None,
        });
        self.search_results.clear();
        self.search_query = SharedString::from("");
        cx.notify();
    }

    fn calculate_profitability(&mut self, cx: &mut Context<Self>) {
        let Some(ref selected) = self.selected_item else {
            return;
        };

        let item_id = selected.item_id;
        self.calculating = true;
        cx.notify();

        let crafter_service = self.crafter_service.clone();
        let this = cx.entity().downgrade();
        gpui_tokio::Tokio::spawn(cx, async move {
            if let Ok(result) = crafter_service.read(&cx).calculate_profitability(item_id).await {
                let _ = this.update(cx, |this: &mut Self, cx: &mut Context<Self>| {
                    if let Some(ref mut selected) = this.selected_item {
                        selected.result = Some(result.clone());
                    }
                    this.calculating = false;
                    cx.notify();
                });
                // Add to history
                let _ = crafter_service.update(cx, |service, cx| {
                    service.add_to_history(&result, cx);
                });
            } else {
                let _ = this.update(cx, |this: &mut Self, cx: &mut Context<Self>| {
                    this.calculating = false;
                    cx.notify();
                });
            }
        })
        .detach();
    }

    fn set_market_price(&mut self, price: f64, cx: &mut Context<Self>) {
        let Some(ref selected) = self.selected_item else {
            return;
        };

        self.crafter_service.update(cx, |service, cx| {
            service.set_market_price(selected.item_id, selected.item_name.clone(), price, cx);
        });
    }

    fn set_resource_price(&mut self, resource_id: i32, price: f64, cx: &mut Context<Self>) {
        let Some(ref selected) = self.selected_item else {
            return;
        };

        self.crafter_service.update(cx, |service, cx| {
            service.set_resource_price(
                selected.item_id,
                selected.item_name.clone(),
                resource_id,
                price,
                cx,
            );
        });
    }
}

impl Render for CrafterWidget {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id("crafter-widget")
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.bg)
            .p_4()
            .gap_4()
            .child(
                // Header
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .text_size(gpui::px(20.))
                            .text_color(theme.text)
                            .child("Crafter - Calculateur de Rentabilité"),
                    ),
            )
            .child(
                // Search bar
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_size(gpui::px(14.))
                            .text_color(theme.text_muted)
                            .child("Rechercher un item à crafter:"),
                    )
                    .child(
                        div()
                            .w_full()
                            .h(gpui::px(40.))
                            .bg(theme.surface)
                            .rounded(gpui::px(8.))
                            .border_1()
                            .border_color(theme.accent_alt.opacity(0.3))
                            .px_3()
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .text_color(theme.text)
                                    .child(self.search_query.clone()),
                            ),
                    )
                    .when(!self.search_results.is_empty(), |this| {
                        this.child(
                            div()
                                .w_full()
                                .max_h(gpui::px(200.))
                                .bg(theme.surface)
                                .rounded(gpui::px(8.))
                                .border_1()
                                .border_color(theme.accent_alt.opacity(0.3))
                                .overflow_hidden()
                                .children(self.search_results.iter().map(|item| {
                                    let item_id = item.id;
                                    let item_name = item.name.clone();
                                    div()
                                        .w_full()
                                        .h(gpui::px(36.))
                                        .px_3()
                                        .flex()
                                        .items_center()
                                        .hover(|style| style.bg(theme.hover))
                                        .cursor_pointer()
                                        .on_mouse_down(MouseButton::Left, cx.listener(move |this, _event, _window, cx| {
                                            this.select_item(item_id, item_name.clone(), cx);
                                        }))
                                        .child(
                                            div()
                                                .text_color(theme.text)
                                                .child(item.name.clone()),
                                        )
                                })),
                        )
                    }),
            )
            .when(self.selected_item.is_some(), |this| {
                let selected = self.selected_item.as_ref().unwrap();
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_4()
                        .child(
                            // Selected item header
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(gpui::px(16.))
                                        .text_color(theme.accent)
                                        .child(format!("Item: {}", selected.item_name)),
                                )
                                .child(
                                    div()
                                        .px_4()
                                        .py_2()
                                        .bg(theme.accent)
                                        .rounded(gpui::px(6.))
                                        .cursor_pointer()
                                        .hover(|style| style.bg(theme.accent.opacity(0.8)))
                                        .on_mouse_down(MouseButton::Left, cx.listener(|this, _event, _window, cx| {
                                            this.calculate_profitability(cx);
                                        }))
                                        .child(
                                            div()
                                                .text_color(theme.bg)
                                                .child(if self.calculating {
                                                    "Calcul en cours..."
                                                } else {
                                                    "Calculer"
                                                }),
                                        ),
                                ),
                        )
                        .when(selected.result.is_some(), |this| {
                            let result = selected.result.as_ref().unwrap();
                            this.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_3()
                                    .child(
                                        // Results summary
                                        div()
                                            .p_4()
                                            .bg(theme.surface)
                                            .rounded(gpui::px(8.))
                                            .border_1()
                                            .border_color(if result.is_profitable {
                                                theme.green
                                            } else {
                                                theme.red
                                            })
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .flex()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .text_color(theme.text_muted)
                                                            .child("Coût de craft:"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_color(theme.text)
                                                            .child(format!("{:.0} K", result.craft_cost)),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .text_color(theme.text_muted)
                                                            .child("Prix HDV:"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_color(theme.text)
                                                            .child(format!("{:.0} K", result.market_price)),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .text_color(theme.text_muted)
                                                            .child("Profit:"),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_color(if result.is_profitable {
                                                                theme.green
                                                            } else {
                                                                theme.red
                                                            })
                                                            .child(format!(
                                                                "{:.0} K ({:.1}%)",
                                                                result.profit, result.profit_margin
                                                            )),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        // Ingredients list
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap_2()
                                            .child(
                                                div()
                                                    .text_size(gpui::px(14.))
                                                    .text_color(theme.text)
                                                    .child("Ressources nécessaires:"),
                                            )
                                            .children(result.ingredients.iter().map(|ingredient| {
                                                div()
                                                    .p_3()
                                                    .bg(theme.surface)
                                                    .rounded(gpui::px(6.))
                                                    .flex()
                                                    .justify_between()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .gap_2()
                                                            .child(
                                                                div()
                                                                    .text_color(theme.text)
                                                                    .child(format!(
                                                                        "{}x {}",
                                                                        ingredient.quantity,
                                                                        ingredient.item_name
                                                                    )),
                                                            ),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_color(theme.text_muted)
                                                            .child(format!(
                                                                "{:.0} K/u = {:.0} K",
                                                                ingredient.unit_price,
                                                                ingredient.total_cost
                                                            )),
                                                    )
                                            })),
                                    ),
                            )
                        }),
                )
            })
            .when(self.selected_item.is_none(), |this| {
                this.child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .text_color(theme.text_muted)
                                .text_size(gpui::px(16.))
                                .child("Recherchez un item pour commencer"),
                        ),
                )
            })
    }
}
