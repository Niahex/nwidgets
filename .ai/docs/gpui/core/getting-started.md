# Getting Started with GPUI

## Installation

Add GPUI to your `Cargo.toml`:

```toml
[dependencies]
gpui = "0.2.2"

# Optional features
gpui = { version = "0.2.2", features = ["wayland", "x11"] }
```

## System Dependencies

### NixOS
```nix
# Add to your configuration.nix or shell.nix
{ pkgs, ... }:
{
  environment.systemPackages = with pkgs; [
    # Wayland dependencies
    wayland
    wayland-protocols
    libxkbcommon
    
    # Graphics dependencies
    vulkan-loader
    vulkan-validation-layers
    
    # X11 dependencies (fallback)
    xorg.libX11
    xorg.libXrandr
    xorg.libXi
    
    # Build dependencies
    pkg-config
    cmake
  ];
}
```

Or for development shell:
```nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    wayland
    libxkbcommon
    vulkan-loader
    pkg-config
  ];
}
```

## Hello World Example

```rust
use gpui::{
    App, Application, Bounds, Context, SharedString, Window, 
    WindowBounds, WindowOptions, div, prelude::*, px, rgb, size,
};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(rgb(0x505050))
            .size(px(500.0))
            .justify_center()
            .items_center()
            .shadow_lg()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!("Hello, {}!", &self.text))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                cx.new(|_| HelloWorld {
                    text: "World".into(),
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
```

## Building and Running

```bash
# Debug build
cargo run

# Release build
cargo run --release

# With specific features
cargo run --features "wayland,x11"
```

## Core Concepts

### Application Lifecycle

Every GPUI application starts with an `Application` instance:

```rust
Application::new().run(|cx: &mut App| {
    // Application initialization
    cx.open_window(/* ... */);
    cx.activate(true);
});
```

### Entities and State

Entities are the fundamental unit of state management:

```rust
struct Counter {
    value: i32,
}

// Create an entity
let counter: Model<Counter> = cx.new(|_| Counter { value: 0 });
```

### Views and Rendering

Views are entities that can be rendered:

```rust
impl Render for MyView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().child("Hello World")
    }
}
```

## Next Steps

1. Read the [Architecture](architecture.md) documentation
2. Explore the [API Reference](../api/README.md)
3. Try the [Examples](../examples/README.md)
4. Learn about [Advanced Topics](../advanced/README.md)
