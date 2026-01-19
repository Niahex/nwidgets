# Elements API

## Div Element

The primary container element with full styling support.

```rust
pub fn div() -> Div
```

### Layout Methods

```rust
impl Div {
    // Flexbox
    pub fn flex(self) -> Self
    pub fn flex_col(self) -> Self
    pub fn flex_row(self) -> Self
    pub fn justify_center(self) -> Self
    pub fn justify_start(self) -> Self
    pub fn justify_end(self) -> Self
    pub fn justify_between(self) -> Self
    pub fn items_center(self) -> Self
    pub fn items_start(self) -> Self
    pub fn items_end(self) -> Self
    
    // Gap and spacing
    pub fn gap(self, gap: impl Into<Pixels>) -> Self
    pub fn gap_1(self) -> Self
    pub fn gap_2(self) -> Self
    pub fn gap_3(self) -> Self
    pub fn gap_4(self) -> Self
}
```

### Sizing Methods

```rust
impl Div {
    pub fn size(self, size: impl Into<Size<Pixels>>) -> Self
    pub fn w(self, width: impl Into<Pixels>) -> Self
    pub fn h(self, height: impl Into<Pixels>) -> Self
    pub fn w_full(self) -> Self
    pub fn h_full(self) -> Self
    pub fn min_w(self, width: impl Into<Pixels>) -> Self
    pub fn max_w(self, width: impl Into<Pixels>) -> Self
    pub fn min_h(self, height: impl Into<Pixels>) -> Self
    pub fn max_h(self, height: impl Into<Pixels>) -> Self
}
```

### Spacing Methods

```rust
impl Div {
    // Padding
    pub fn p(self, padding: impl Into<Pixels>) -> Self
    pub fn px(self, padding: impl Into<Pixels>) -> Self
    pub fn py(self, padding: impl Into<Pixels>) -> Self
    pub fn pt(self, padding: impl Into<Pixels>) -> Self
    pub fn pr(self, padding: impl Into<Pixels>) -> Self
    pub fn pb(self, padding: impl Into<Pixels>) -> Self
    pub fn pl(self, padding: impl Into<Pixels>) -> Self
    
    // Margin
    pub fn m(self, margin: impl Into<Pixels>) -> Self
    pub fn mx(self, margin: impl Into<Pixels>) -> Self
    pub fn my(self, margin: impl Into<Pixels>) -> Self
    pub fn mt(self, margin: impl Into<Pixels>) -> Self
    pub fn mr(self, margin: impl Into<Pixels>) -> Self
    pub fn mb(self, margin: impl Into<Pixels>) -> Self
    pub fn ml(self, margin: impl Into<Pixels>) -> Self
    pub fn mx_auto(self) -> Self
}
```

### Color Methods

```rust
impl Div {
    pub fn bg(self, color: impl Into<Hsla>) -> Self
    pub fn text_color(self, color: impl Into<Hsla>) -> Self
    pub fn border_color(self, color: impl Into<Hsla>) -> Self
}
```

### Border Methods

```rust
impl Div {
    pub fn border(self, border: impl Into<Pixels>) -> Self
    pub fn border_1(self) -> Self
    pub fn border_2(self) -> Self
    pub fn border_t(self, border: impl Into<Pixels>) -> Self
    pub fn border_r(self, border: impl Into<Pixels>) -> Self
    pub fn border_b(self, border: impl Into<Pixels>) -> Self
    pub fn border_l(self, border: impl Into<Pixels>) -> Self
    pub fn rounded(self, radius: impl Into<Pixels>) -> Self
    pub fn rounded_md(self) -> Self
    pub fn rounded_lg(self) -> Self
    pub fn rounded_full(self) -> Self
}
```

### Effect Methods

```rust
impl Div {
    pub fn shadow_lg(self) -> Self
    pub fn shadow_md(self) -> Self
    pub fn shadow_sm(self) -> Self
    pub fn opacity(self, opacity: f32) -> Self
}
```

### Event Methods

```rust
impl Div {
    pub fn on_click<V>(self, handler: impl Fn(&ClickEvent, &mut Context<V>) + 'static) -> Self
    pub fn on_mouse_down<V>(self, button: MouseButton, handler: impl Fn(&MouseDownEvent, &mut Context<V>) + 'static) -> Self
    pub fn on_mouse_up<V>(self, button: MouseButton, handler: impl Fn(&MouseUpEvent, &mut Context<V>) + 'static) -> Self
    pub fn on_mouse_move<V>(self, handler: impl Fn(&MouseMoveEvent, &mut Context<V>) + 'static) -> Self
    pub fn on_key_down<V>(self, handler: impl Fn(&KeyDownEvent, &mut Context<V>) + 'static) -> Self
    pub fn on_key_up<V>(self, handler: impl Fn(&KeyUpEvent, &mut Context<V>) + 'static) -> Self
}
```

### Content Methods

```rust
impl Div {
    pub fn child(self, child: impl IntoElement) -> Self
    pub fn children(self, children: impl IntoIterator<Item = impl IntoElement>) -> Self
    pub fn when(self, condition: bool, then: impl FnOnce(Self) -> Self) -> Self
    pub fn when_some<T>(self, option: Option<T>, then: impl FnOnce(Self, T) -> Self) -> Self
}
```

## Text Element

For rendering text content.

```rust
pub fn text(content: impl Into<SharedString>) -> Text
```

### Text Methods

```rust
impl Text {
    pub fn size(self, size: impl Into<Pixels>) -> Self
    pub fn color(self, color: impl Into<Hsla>) -> Self
    pub fn weight(self, weight: FontWeight) -> Self
    pub fn italic(self) -> Self
    pub fn underline(self) -> Self
    pub fn line_through(self) -> Self
}
```

### Font Weights

```rust
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}
```

## Image Element

For displaying images.

```rust
pub fn img(source: impl Into<ImageSource>) -> Img
```

### Image Methods

```rust
impl Img {
    pub fn size(self, size: impl Into<Size<Pixels>>) -> Self
    pub fn w(self, width: impl Into<Pixels>) -> Self
    pub fn h(self, height: impl Into<Pixels>) -> Self
    pub fn object_fit(self, fit: ObjectFit) -> Self
}
```

### Image Sources

```rust
pub enum ImageSource {
    File(Arc<Path>),
    Uri(SharedString),
    Data(Arc<[u8]>),
}
```

### Object Fit

```rust
pub enum ObjectFit {
    Fill,
    Contain,
    Cover,
    ScaleDown,
    None,
}
```

## Example Usage

```rust
use gpui::{div, text, img, px, rgb, ImageSource};

div()
    .flex()
    .flex_col()
    .gap_4()
    .p_8()
    .bg(rgb(0xffffff))
    .border_1()
    .border_color(rgb(0xe5e7eb))
    .rounded_lg()
    .shadow_md()
    .child(
        text("Hello World")
            .size(px(24.))
            .weight(FontWeight::Bold)
            .color(rgb(0x1f2937))
    )
    .child(
        img(ImageSource::File("image.jpg".into()))
            .size(px(200.))
            .object_fit(ObjectFit::Cover)
            .rounded_md()
    )
    .on_click(cx.listener(|this, event, cx| {
        // Handle click
    }))
```
