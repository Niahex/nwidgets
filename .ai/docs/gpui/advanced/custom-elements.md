# Custom Elements

Creating reusable UI components with custom elements.

## Basic Custom Element

```rust
use gpui::{Element, IntoElement, Bounds, Context, Pixels};

struct CustomButton {
    label: SharedString,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Context<V>) + 'static>>,
}

impl CustomButton {
    fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
    }
    
    fn on_click<V>(mut self, handler: impl Fn(&ClickEvent, &mut Context<V>) + 'static) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl<V: 'static> Element<V> for CustomButton {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn request_layout(&mut self, cx: &mut Context<V>) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        let layout_id = cx.request_layout(style, []);
        (layout_id, ())
    }

    fn paint(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<V>) {
        cx.paint_quad(Quad {
            bounds,
            background: Some(rgb(0x3b82f6)),
            border_radius: px(6.).into(),
            ..Default::default()
        });
        
        cx.paint_text(
            bounds.center(),
            self.label.clone(),
            rgb(0xffffff),
        );
    }
}

impl<V> IntoElement<V> for CustomButton {
    type Element = Self;
    
    fn into_element(self) -> Self::Element {
        self
    }
}
```

## Usage

```rust
CustomButton::new("Click Me")
    .on_click(|event, cx| {
        println!("Button clicked!");
    })
```

## Advanced Patterns

### Stateful Elements
- Internal state management
- Event handling
- Custom painting

### Composition
- Combining multiple elements
- Reusable component libraries
- Theme integration
