# API Reference Index

Complete API documentation for GPUI framework.

## Core APIs

### [Application](application.md)
- `Application` - Main application instance
- `App` - Application context
- `WindowOptions` - Window configuration
- `WindowHandle` - Window management

### [Context](context.md)
- `Context<T>` - Entity operation context
- `Model<T>` - Entity smart pointer
- Global state management
- Event handling

### [Elements](elements.md)
- `div()` - Container element
- `text()` - Text rendering
- `img()` - Image display
- Layout and styling methods

### [Events](events.md)
- Mouse events (`ClickEvent`, `MouseDownEvent`, etc.)
- Keyboard events (`KeyDownEvent`, `KeyUpEvent`)
- Actions and key bindings
- Focus management

### [Styling](styling.md)
- Color system (`rgb`, `hsl`, predefined colors)
- Layout (Flexbox, Grid, positioning)
- Sizing and spacing
- Typography and effects

## Rendering System

### Element Trait
```rust
pub trait Element<V> {
    type RequestLayoutState;
    type PrepaintState;
    
    fn request_layout(&mut self, cx: &mut Context<V>) -> (LayoutId, Self::RequestLayoutState);
    fn paint(&mut self, bounds: Bounds<Pixels>, cx: &mut Context<V>);
}
```

### IntoElement Trait
```rust
pub trait IntoElement<V> {
    type Element: Element<V>;
    fn into_element(self) -> Self::Element;
}
```

### Render Trait
```rust
pub trait Render {
    fn render(&mut self, cx: &mut Context<Self>) -> impl IntoElement;
}
```

## State Management

### Entity System
- `Model<T>` - Managed entity reference
- `WeakModel<T>` - Weak entity reference
- `EntityId` - Unique entity identifier

### Global State
- `Global` trait for global state types
- `cx.global::<T>()` - Read global state
- `cx.set_global(value)` - Set global state

### Subscriptions
- `Subscription` - Event subscription handle
- `EventEmitter<E>` - Event emission trait
- `cx.subscribe()` - Subscribe to events

## Async System

### Tasks
- `Task<T>` - Async task handle
- `cx.spawn()` - Spawn async task
- `BackgroundExecutor` - Background thread pool
- `ForegroundExecutor` - Main thread executor

### Futures
- Integration with standard Rust futures
- Automatic cleanup on drop
- Context propagation

## Platform Integration

### Window Management
- Multi-window support
- Platform-specific features
- Display management

### Input Handling
- Mouse and keyboard events
- Touch and gesture support
- Accessibility integration

### Graphics
- GPU-accelerated rendering
- Cross-platform graphics APIs
- High DPI support

## Testing

### Test Context
- `TestAppContext` - Testing environment
- `#[gpui::test]` - Test macro
- Mock implementations

### Visual Testing
- Screenshot comparison
- Layout verification
- Interaction simulation
