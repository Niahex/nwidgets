# Events API

## Mouse Events

### ClickEvent

```rust
pub struct ClickEvent {
    pub position: Point<Pixels>,
    pub button: MouseButton,
    pub modifiers: Modifiers,
    pub click_count: usize,
}
```

### MouseDownEvent

```rust
pub struct MouseDownEvent {
    pub position: Point<Pixels>,
    pub button: MouseButton,
    pub modifiers: Modifiers,
    pub click_count: usize,
}
```

### MouseUpEvent

```rust
pub struct MouseUpEvent {
    pub position: Point<Pixels>,
    pub button: MouseButton,
    pub modifiers: Modifiers,
}
```

### MouseMoveEvent

```rust
pub struct MouseMoveEvent {
    pub position: Point<Pixels>,
    pub pressed_button: Option<MouseButton>,
    pub modifiers: Modifiers,
}
```

### MouseButton

```rust
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Navigate(NavigationDirection),
}

pub enum NavigationDirection {
    Back,
    Forward,
}
```

## Keyboard Events

### KeyDownEvent

```rust
pub struct KeyDownEvent {
    pub keystroke: Keystroke,
    pub is_held: bool,
}
```

### KeyUpEvent

```rust
pub struct KeyUpEvent {
    pub keystroke: Keystroke,
}
```

### Keystroke

```rust
pub struct Keystroke {
    pub modifiers: Modifiers,
    pub key: String,
    pub ime_key: Option<String>,
}
```

### Modifiers

```rust
pub struct Modifiers {
    pub control: bool,
    pub alt: bool,
    pub shift: bool,
    pub super: bool,    // Super key on Linux
    pub function: bool,
}
```

## Actions

Actions are user-defined structs for keyboard shortcuts and commands.

### Action Trait

```rust
pub trait Action: Clone + PartialEq + Eq + 'static {
    fn name() -> &'static str;
}
```

### Defining Actions

```rust
#[derive(Clone, PartialEq, Eq)]
struct Increment;

#[derive(Clone, PartialEq, Eq)]
struct Decrement;

impl Action for Increment {
    fn name() -> &'static str { "increment" }
}

impl Action for Decrement {
    fn name() -> &'static str { "decrement" }
}
```

### Key Bindings

```rust
pub struct KeyBinding {
    pub keystroke: Keystroke,
    pub action: Box<dyn Action>,
    pub context: Option<KeyContext>,
}

impl KeyBinding {
    pub fn new<A: Action>(
        keystroke: impl TryInto<Keystroke>,
        action: A,
        context: Option<KeyContext>,
    ) -> Self
}
```

### Registering Key Bindings

```rust
cx.bind_keys([
    KeyBinding::new("ctrl-=", Increment, None),
    KeyBinding::new("ctrl--", Decrement, None),
    KeyBinding::new("ctrl-c", Copy, Some("Editor")),
]);
```

## Event Handling Examples

### Mouse Events

```rust
div()
    .on_click(cx.listener(|this, event, cx| {
        println!("Clicked at {:?}", event.position);
        if event.modifiers.shift {
            println!("Shift was held");
        }
    }))
    .on_mouse_down(MouseButton::Right, cx.listener(|this, event, cx| {
        println!("Right mouse button pressed");
    }))
    .on_mouse_move(cx.listener(|this, event, cx| {
        if let Some(button) = event.pressed_button {
            println!("Dragging with {:?}", button);
        }
    }))
```

### Keyboard Events

```rust
div()
    .on_key_down(cx.listener(|this, event, cx| {
        match event.keystroke.key.as_str() {
            "Enter" => this.submit(cx),
            "Escape" => this.cancel(cx),
            "ArrowUp" => this.move_up(cx),
            "ArrowDown" => this.move_down(cx),
            _ => {}
        }
        
        if event.keystroke.modifiers.control && event.keystroke.key == "s" {
            this.save(cx);
        }
    }))
```

### Action Handling

```rust
impl MyView {
    fn handle_action(&mut self, action: &dyn Action, cx: &mut Context<Self>) {
        if let Some(increment) = action.downcast_ref::<Increment>() {
            self.counter += 1;
            cx.notify();
        } else if let Some(decrement) = action.downcast_ref::<Decrement>() {
            self.counter -= 1;
            cx.notify();
        }
    }
}
```

## Focus Events

### FocusHandle

```rust
pub struct FocusHandle {
    // Internal implementation
}

impl FocusHandle {
    pub fn is_focused(&self, cx: &Context<impl Any>) -> bool
    pub fn focus(&self, cx: &mut Context<impl Any>)
    pub fn blur(&self, cx: &mut Context<impl Any>)
}
```

### Focus Management

```rust
struct FocusableView {
    focus_handle: FocusHandle,
}

impl Render for FocusableView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .when(self.focus_handle.is_focused(cx), |div| {
                div.border_color(rgb(0x3b82f6))
            })
            .on_key_down(cx.listener(|this, event, cx| {
                if this.focus_handle.is_focused(cx) {
                    // Handle key events when focused
                }
            }))
    }
}
```

## Event Propagation

Events in GPUI follow a capture/bubble pattern:

1. **Capture Phase**: Events travel down the element tree
2. **Target Phase**: Event reaches the target element
3. **Bubble Phase**: Events travel up the element tree

### Stopping Propagation

```rust
div()
    .on_click(cx.listener(|this, event, cx| {
        // Handle event and stop propagation
        cx.stop_propagation();
    }))
```
