# GPUI LayerShell - Complete Documentation

## Overview

LayerShell is a Wayland protocol extension that allows applications to create surfaces that are positioned relative to the screen edges, rather than being managed as regular windows. This is essential for creating desktop widgets, panels, notifications, wallpapers, and overlay applications.

## What is LayerShell?

LayerShell (`zwlr_layer_shell_v1`) is a Wayland protocol that provides:

- **Screen-anchored surfaces**: Attach to screen edges (top, bottom, left, right)
- **Layered rendering**: Multiple layers (Background, Bottom, Top, Overlay)
- **Exclusive zones**: Reserve screen space that other windows avoid
- **Desktop integration**: Perfect for panels, docks, notifications, widgets

## Core Types

### Layer

Defines which layer the surface is rendered on:

```rust
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Layer {
    /// The background layer, typically used for wallpapers
    Background,
    
    /// The bottom layer
    Bottom,
    
    /// The top layer, typically used for fullscreen windows
    Top,
    
    /// The overlay layer, used for surfaces that should always be on top
    #[default]
    Overlay,
}
```

**Layer Hierarchy** (bottom to top):
1. `Background` - Wallpapers, desktop backgrounds
2. `Bottom` - Desktop widgets, always-visible panels
3. `Top` - Fullscreen applications, important overlays
4. `Overlay` - Critical notifications, system dialogs

### Anchor

Screen anchor points using bitflags for flexible positioning:

```rust
bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct Anchor: u32 {
        /// Anchor to the top edge of the screen
        const TOP = 1;
        /// Anchor to the bottom edge of the screen
        const BOTTOM = 2;
        /// Anchor to the left edge of the screen
        const LEFT = 4;
        /// Anchor to the right edge of the screen
        const RIGHT = 8;
    }
}
```

**Anchor Combinations**:
- `Anchor::TOP` - Top edge only
- `Anchor::LEFT | Anchor::RIGHT` - Stretch across width
- `Anchor::TOP | Anchor::LEFT` - Top-left corner
- `Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT` - Left edge, full height

### KeyboardInteractivity

Controls how keyboard input is handled:

```rust
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum KeyboardInteractivity {
    /// No keyboard inputs delivered, cannot receive focus
    None,
    
    /// Exclusive keyboard focus when above shell surface layer
    Exclusive,
    
    /// Can be focused like a normal window
    #[default]
    OnDemand,
}
```

**Use Cases**:
- `None` - Status bars, wallpapers, non-interactive widgets
- `Exclusive` - Lock screens, critical system dialogs
- `OnDemand` - Interactive panels, application launchers

### LayerShellOptions

Complete configuration for LayerShell windows:

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LayerShellOptions {
    /// Namespace for compositor rules (cannot be changed after creation)
    pub namespace: String,
    
    /// Rendering layer
    pub layer: Layer,
    
    /// Screen anchor points
    pub anchor: Anchor,
    
    /// Exclusive zone size (reserves screen space)
    pub exclusive_zone: Option<Pixels>,
    
    /// Exclusive zone anchor (defaults to anchor if unspecified)
    pub exclusive_edge: Option<Anchor>,
    
    /// Margins from anchor points (top, right, bottom, left)
    pub margin: Option<(Pixels, Pixels, Pixels, Pixels)>,
    
    /// Keyboard input handling
    pub keyboard_interactivity: KeyboardInteractivity,
}
```

## Creating LayerShell Windows

### Basic LayerShell Window

```rust
use gpui::{
    App, Application, WindowOptions, WindowKind, layer_shell::*,
    px, Bounds, Size, point,
};

fn create_layer_shell_window(cx: &mut App) {
    cx.open_window(
        WindowOptions {
            kind: WindowKind::LayerShell(LayerShellOptions {
                namespace: "my-app".to_string(),
                layer: Layer::Overlay,
                anchor: Anchor::TOP | Anchor::RIGHT,
                margin: Some((px(10.), px(10.), px(0.), px(0.))),
                keyboard_interactivity: KeyboardInteractivity::None,
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(Bounds {
                origin: point(px(0.), px(0.)),
                size: Size::new(px(300.), px(100.)),
            })),
            ..Default::default()
        },
        |_, cx| cx.new(|_| MyWidget::new()),
    ).unwrap();
}
```

### Desktop Panel Example

```rust
// Top panel that reserves screen space
WindowKind::LayerShell(LayerShellOptions {
    namespace: "desktop-panel".to_string(),
    layer: Layer::Top,
    anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
    exclusive_zone: Some(px(40.)), // Reserve 40px at top
    margin: None,
    keyboard_interactivity: KeyboardInteractivity::OnDemand,
})
```

### Notification Widget

```rust
// Floating notification in top-right corner
WindowKind::LayerShell(LayerShellOptions {
    namespace: "notifications".to_string(),
    layer: Layer::Overlay,
    anchor: Anchor::TOP | Anchor::RIGHT,
    exclusive_zone: None, // Don't reserve space
    margin: Some((px(10.), px(10.), px(0.), px(0.))),
    keyboard_interactivity: KeyboardInteractivity::None,
})
```

### Desktop Widget

```rust
// Bottom-left desktop widget
WindowKind::LayerShell(LayerShellOptions {
    namespace: "desktop-widget".to_string(),
    layer: Layer::Bottom,
    anchor: Anchor::BOTTOM | Anchor::LEFT,
    exclusive_zone: None,
    margin: Some((px(0.), px(0.), px(20.), px(20.))),
    keyboard_interactivity: KeyboardInteractivity::OnDemand,
})
```

### Wallpaper Application

```rust
// Full-screen wallpaper
WindowKind::LayerShell(LayerShellOptions {
    namespace: "wallpaper".to_string(),
    layer: Layer::Background,
    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
    exclusive_zone: None,
    margin: None,
    keyboard_interactivity: KeyboardInteractivity::None,
})
```

## Complete Example: Digital Clock Widget

```rust
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use gpui::{
    App, Application, Bounds, Context, FontWeight, Size, Window, 
    WindowBackgroundAppearance, WindowBounds, WindowKind, WindowOptions, 
    div, layer_shell::*, point, prelude::*, px, rems, rgba, white,
};

struct ClockWidget;

impl ClockWidget {
    fn new(cx: &mut Context<Self>) -> Self {
        // Update every second
        cx.spawn(async move |this, cx| {
            loop {
                let _ = this.update(cx, |_, cx| cx.notify());
                cx.background_executor()
                    .timer(Duration::from_millis(1000))
                    .await;
            }
        }).detach();
        
        ClockWidget
    }
}

impl Render for ClockWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let hours = (now / 3600) % 24;
        let minutes = (now / 60) % 60;
        let seconds = now % 60;

        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .text_size(rems(3.0))
            .font_weight(FontWeight::BOLD)
            .text_color(white())
            .bg(rgba(0x000000aa))
            .rounded_lg()
            .border_1()
            .border_color(rgba(0xffffff44))
            .child(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(
            WindowOptions {
                titlebar: None,
                window_bounds: Some(WindowBounds::Windowed(Bounds {
                    origin: point(px(0.), px(0.)),
                    size: Size::new(px(200.), px(80.)),
                })),
                app_id: Some("clock-widget".to_string()),
                window_background: WindowBackgroundAppearance::Transparent,
                kind: WindowKind::LayerShell(LayerShellOptions {
                    namespace: "clock".to_string(),
                    layer: Layer::Overlay,
                    anchor: Anchor::TOP | Anchor::RIGHT,
                    margin: Some((px(20.), px(20.), px(0.), px(0.))),
                    keyboard_interactivity: KeyboardInteractivity::None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            |_, cx| cx.new(ClockWidget::new),
        ).unwrap();
    });
}
```

## Runtime Configuration

LayerShell surfaces can be reconfigured after creation:

### Window Methods

```rust
impl Window {
    /// Set exclusive zone size (Wayland only)
    pub fn set_exclusive_zone(&self, zone: i32);
    
    /// Set exclusive zone edge (Wayland only)
    pub fn set_exclusive_edge(&self, edge: Anchor);
    
    /// Set margins from anchor points (Wayland only)
    pub fn set_margin(&self, top: i32, right: i32, bottom: i32, left: i32);
    
    /// Set keyboard interactivity mode (Wayland only)
    pub fn set_keyboard_interactivity(&self, interactivity: KeyboardInteractivity);
    
    /// Set input region for touch/mouse events (Wayland only)
    /// Pass None to disable all input, Some(bounds) for specific region
    pub fn set_input_region(&self, region: Option<&Bounds<i32>>);
    
    /// Set rendering layer (Wayland only)
    pub fn set_layer(&self, layer: Layer);
}
```

### Dynamic Updates Example

```rust
use gpui::{Bounds, point, px};

impl MyPanel {
    fn toggle_exclusive_mode(&mut self, window: &Window) {
        if self.exclusive_mode {
            window.set_exclusive_zone(0); // Remove exclusive zone
            window.set_layer(Layer::Overlay);
        } else {
            window.set_exclusive_zone(40); // Reserve 40px
            window.set_layer(Layer::Top);
        }
        self.exclusive_mode = !self.exclusive_mode;
    }
    
    fn set_interactive(&mut self, window: &Window, interactive: bool) {
        let interactivity = if interactive {
            KeyboardInteractivity::OnDemand
        } else {
            KeyboardInteractivity::None
        };
        window.set_keyboard_interactivity(interactivity);
    }
    
    fn update_margins(&mut self, window: &Window, margin: i32) {
        window.set_margin(margin, margin, margin, margin);
    }
    
    fn set_input_area(&mut self, window: &Window, clickable: bool) {
        if clickable {
            // Allow input in specific region
            let bounds = Bounds {
                origin: point(0, 0),
                size: gpui::Size { width: 200, height: 50 },
            };
            window.set_input_region(Some(&bounds));
        } else {
            // Disable all input (click-through)
            window.set_input_region(None);
        }
    }
    
    fn make_click_through(&mut self, window: &Window) {
        // Make window completely click-through
        window.set_input_region(None);
        window.set_keyboard_interactivity(KeyboardInteractivity::None);
    }
}
```

## Compositor Compatibility

### Supported Compositors

| Compositor | LayerShell Support | Notes |
|------------|-------------------|-------|
| **Sway** | ✅ Full | Reference implementation |
| **Hyprland** | ✅ Full | Excellent support |
| **River** | ✅ Full | Complete implementation |
| **Wayfire** | ✅ Full | Good support |
| **GNOME Shell** | ⚠️ Partial | Limited layer shell support |
| **KDE Plasma** | ⚠️ Partial | Basic support |

### Feature Support Matrix

| Feature | Sway | Hyprland | River | GNOME | KDE |
|---------|------|----------|-------|-------|-----|
| All Layers | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| Exclusive Zones | ✅ | ✅ | ✅ | ✅ | ✅ |
| Keyboard Interactivity | ✅ | ✅ | ✅ | ⚠️ | ⚠️ |
| Multiple Anchors | ✅ | ✅ | ✅ | ✅ | ✅ |
| Margins | ✅ | ✅ | ✅ | ✅ | ✅ |

## Error Handling

### LayerShellNotSupportedError

```rust
use gpui::layer_shell::LayerShellNotSupportedError;

match cx.open_window(window_options, view_builder) {
    Ok(window) => {
        // LayerShell window created successfully
    }
    Err(e) => {
        if e.downcast_ref::<LayerShellNotSupportedError>().is_some() {
            eprintln!("Compositor doesn't support LayerShell");
            // Fallback to normal window
            create_normal_window(cx);
        }
    }
}
```

### Graceful Fallback

```rust
fn create_widget_window(cx: &mut App) -> Result<WindowHandle<MyWidget>, Box<dyn std::error::Error>> {
    // Try LayerShell first
    let layer_shell_options = WindowOptions {
        kind: WindowKind::LayerShell(LayerShellOptions::default()),
        ..Default::default()
    };
    
    match cx.open_window(layer_shell_options, |_, cx| cx.new(MyWidget::new)) {
        Ok(window) => Ok(window),
        Err(e) if e.downcast_ref::<LayerShellNotSupportedError>().is_some() => {
            // Fallback to normal window
            let normal_options = WindowOptions {
                kind: WindowKind::Normal,
                ..Default::default()
            };
            cx.open_window(normal_options, |_, cx| cx.new(MyWidget::new))
                .map_err(Into::into)
        }
        Err(e) => Err(e.into()),
    }
}
```

## Advanced Features (Fork Extensions)

### Input Region Control

Control which parts of your LayerShell surface receive input events:

```rust
use gpui::{Bounds, point, Size};

struct InteractiveWidget {
    click_through_mode: bool,
}

impl InteractiveWidget {
    fn toggle_click_through(&mut self, window: &Window) {
        self.click_through_mode = !self.click_through_mode;
        
        if self.click_through_mode {
            // Make entire window click-through
            window.set_input_region(None);
        } else {
            // Only button area receives input
            let button_area = Bounds {
                origin: point(10, 10),
                size: Size { width: 100, height: 30 },
            };
            window.set_input_region(Some(&button_area));
        }
    }
    
    fn set_partial_input(&mut self, window: &Window) {
        // Multiple input regions (combine bounds as needed)
        let interactive_bounds = Bounds {
            origin: point(0, 0),
            size: Size { width: 200, height: 50 },
        };
        window.set_input_region(Some(&interactive_bounds));
    }
}
```

### Dynamic Margin Adjustment

Adjust margins at runtime for responsive layouts:

```rust
impl ResponsivePanel {
    fn adapt_to_screen_size(&mut self, window: &Window, screen_width: i32) {
        let margin = if screen_width > 1920 {
            20 // Large screens: more margin
        } else if screen_width > 1280 {
            10 // Medium screens: moderate margin
        } else {
            5  // Small screens: minimal margin
        };
        
        window.set_margin(margin, margin, margin, margin);
    }
    
    fn set_asymmetric_margins(&mut self, window: &Window) {
        // Different margins for each side
        window.set_margin(
            10, // top
            20, // right  
            5,  // bottom
            15, // left
        );
    }
}
```

### Layer Switching

Dynamically change layers based on application state:

```rust
impl AdaptiveOverlay {
    fn set_priority_mode(&mut self, window: &Window, high_priority: bool) {
        if high_priority {
            window.set_layer(Layer::Overlay);
            window.set_keyboard_interactivity(KeyboardInteractivity::Exclusive);
        } else {
            window.set_layer(Layer::Top);
            window.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
        }
    }
    
    fn enter_background_mode(&mut self, window: &Window) {
        window.set_layer(Layer::Background);
        window.set_input_region(None); // Click-through
        window.set_keyboard_interactivity(KeyboardInteractivity::None);
    }
}
```

## Best Practices
```rust
// Good: Descriptive, unique namespaces
LayerShellOptions {
    namespace: "com.myapp.panel".to_string(),
    // ...
}

// Bad: Generic names that might conflict
LayerShellOptions {
    namespace: "window".to_string(),
    // ...
}
```

### 2. Layer Selection
```rust
// Desktop panels and docks
layer: Layer::Top,

// Desktop widgets and clocks
layer: Layer::Bottom,

// Wallpapers and backgrounds
layer: Layer::Background,

// Critical notifications and dialogs
layer: Layer::Overlay,
```

### 3. Exclusive Zones
```rust
// Reserve space for panels
LayerShellOptions {
    anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
    exclusive_zone: Some(px(40.)), // Panel height
    // ...
}

// Don't reserve space for widgets
LayerShellOptions {
    anchor: Anchor::BOTTOM | Anchor::RIGHT,
    exclusive_zone: None, // Floating widget
    // ...
}
```

### 4. Keyboard Interactivity
```rust
// Interactive panels and launchers
keyboard_interactivity: KeyboardInteractivity::OnDemand,

// Status displays and clocks
keyboard_interactivity: KeyboardInteractivity::None,

// Lock screens and critical dialogs
keyboard_interactivity: KeyboardInteractivity::Exclusive,
```

### 5. Transparent Backgrounds
```rust
WindowOptions {
    window_background: WindowBackgroundAppearance::Transparent,
    titlebar: None, // Remove titlebar for LayerShell
    // ...
}
```

## Common Use Cases

### System Panel
```rust
LayerShellOptions {
    namespace: "system-panel".to_string(),
    layer: Layer::Top,
    anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
    exclusive_zone: Some(px(32.)),
    keyboard_interactivity: KeyboardInteractivity::OnDemand,
    ..Default::default()
}
```

### Notification Area
```rust
LayerShellOptions {
    namespace: "notifications".to_string(),
    layer: Layer::Overlay,
    anchor: Anchor::TOP | Anchor::RIGHT,
    margin: Some((px(10.), px(10.), px(0.), px(0.))),
    keyboard_interactivity: KeyboardInteractivity::None,
    ..Default::default()
}
```

### Desktop Dock
```rust
LayerShellOptions {
    namespace: "dock".to_string(),
    layer: Layer::Top,
    anchor: Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
    exclusive_zone: Some(px(48.)),
    margin: Some((px(0.), px(10.), px(10.), px(10.))),
    keyboard_interactivity: KeyboardInteractivity::OnDemand,
    ..Default::default()
}
```

### Screen Overlay
```rust
LayerShellOptions {
    namespace: "overlay".to_string(),
    layer: Layer::Overlay,
    anchor: Anchor::TOP | Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
    exclusive_zone: None,
    keyboard_interactivity: KeyboardInteractivity::Exclusive,
    ..Default::default()
}
```

## NixOS Configuration

### System Configuration
```nix
# configuration.nix
{ config, pkgs, ... }:
{
  # Enable Wayland
  services.xserver.displayManager.gdm.wayland = true;
  
  # LayerShell support
  environment.systemPackages = with pkgs; [
    wayland-protocols
    wlr-protocols
  ];
  
  # Compositor with LayerShell support
  programs.sway.enable = true;
  # or
  programs.hyprland.enable = true;
}
```

### Development Shell
```nix
# shell.nix
{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  buildInputs = with pkgs; [
    wayland
    wayland-protocols
    wlr-protocols
    libxkbcommon
    vulkan-loader
  ];
  
  shellHook = ''
    export WAYLAND_DISPLAY=wayland-0
  '';
}
```

LayerShell in GPUI provides powerful desktop integration capabilities for NixOS applications, enabling the creation of panels, widgets, notifications, and other desktop components that integrate seamlessly with Wayland compositors.
