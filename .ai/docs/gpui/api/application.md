# Application API

## Application

The main application instance that manages the event loop and platform integration.

```rust
impl Application {
    pub fn new() -> Self
    pub fn run<F>(self, f: F) where F: FnOnce(&mut App)
}
```

### Example

```rust
use gpui::Application;

fn main() {
    Application::new().run(|cx: &mut App| {
        // Application setup
    });
}
```

## App (Application Context)

The main context for application-level operations.

```rust
impl App {
    pub fn open_window<V>(&mut self, options: WindowOptions, build_view: impl FnOnce(&mut Window, &mut Context<V>) -> V) -> Result<WindowHandle<V>>
    pub fn activate(&mut self, active: bool)
    pub fn quit(&mut self)
    pub fn set_menus(&mut self, menus: Vec<Menu>)
    pub fn bind_keys(&mut self, bindings: impl IntoIterator<Item = KeyBinding>)
}
```

### Methods

#### `open_window`
Creates a new window with the specified options and view.

```rust
cx.open_window(
    WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        ..Default::default()
    },
    |_, cx| {
        cx.new(|_| MyView::new())
    },
)?;
```

#### `activate`
Activates or deactivates the application.

```rust
cx.activate(true); // Required after opening windows
```

#### `quit`
Quits the application.

```rust
cx.quit();
```

#### `set_menus`
Sets the application menu (Linux desktop integration).

```rust
use gpui::{Menu, MenuItem};

let menu = Menu::new("MyApp")
    .entry(MenuItem::action("New", NewFile))
    .separator()
    .entry(MenuItem::action("Quit", Quit));

cx.set_menus(vec![menu]);
```

### Key Bindings

```rust
cx.bind_keys([
    KeyBinding::new("ctrl-=", Increment, None),
    KeyBinding::new("ctrl--", Decrement, None),
]);
```

## WindowOptions

Configuration for window creation.

```rust
pub struct WindowOptions {
    pub window_bounds: Option<WindowBounds>,
    pub titlebar: Option<TitlebarOptions>,
    pub center: bool,
    pub fullscreen: bool,
    pub show: bool,
    pub kind: WindowKind,
    pub is_movable: bool,
    pub display_id: Option<DisplayId>,
}
```

### Example

```rust
let options = WindowOptions {
    window_bounds: Some(WindowBounds::Windowed(
        Bounds::centered(None, size(px(800.0), px(600.0)), cx)
    )),
    titlebar: Some(TitlebarOptions {
        title: Some("My App".into()),
        appears_transparent: false,
        traffic_light_position: None,
    }),
    center: true,
    fullscreen: false,
    show: true,
    kind: WindowKind::Normal,
    is_movable: true,
    display_id: None,
};
```

## WindowHandle

Handle to a window for operations after creation.

```rust
impl<V> WindowHandle<V> {
    pub fn update(&self, cx: &mut App, f: impl FnOnce(&mut V, &mut Context<V>))
    pub fn read(&self, cx: &App) -> &V
    pub fn close(&self, cx: &mut App)
    pub fn minimize(&self, cx: &mut App)
    pub fn zoom(&self, cx: &mut App)
}
```

### Example

```rust
let window_handle = cx.open_window(options, |_, cx| {
    cx.new(|_| MyView::new())
})?;

// Update window content
window_handle.update(cx, |view, cx| {
    view.update_data(cx);
});

// Close window
window_handle.close(cx);
```
