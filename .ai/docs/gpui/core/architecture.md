# GPUI Architecture

## Layered Architecture

GPUI follows a layered architecture with three main registers:

```
┌─────────────────────────────────────────────────────────────┐
│                        Application                          │
├─────────────────────────────────────────────────────────────┤
│  Views & Elements  │  State Management  │  Event System    │
├─────────────────────────────────────────────────────────────┤
│           Platform Abstraction Layer                       │
├─────────────────────────────────────────────────────────────┤
│  Windowing  │  Input  │  Graphics  │  Text  │  Assets      │
├─────────────────────────────────────────────────────────────┤
│                        NixOS                                │
└─────────────────────────────────────────────────────────────┘
```

## Layer 1: State Management (Entities)

- **Entities**: Application state containers owned by GPUI
- **Context**: Access point for GPUI services and state
- **Subscriptions**: Reactive updates and event handling

### Key Components:
- `Model<T>`: Smart pointer to entities
- `Context<T>`: Entity operation context
- `Global`: Application-wide state

## Layer 2: High-Level UI (Views)

- **Views**: Renderable entities implementing the `Render` trait
- **Elements**: Building blocks of the UI tree
- **Styling**: CSS-like styling with Tailwind-inspired API

### Key Components:
- `Render` trait: View rendering interface
- `IntoElement`: Element conversion trait
- `div()`, `text()`, `img()`: Core elements

## Layer 3: Low-Level Rendering (Platform)

- **Platform Abstraction**: Cross-platform windowing and input
- **GPU Rendering**: Hardware-accelerated graphics pipeline
- **Text System**: Advanced text layout and rendering

### Key Components:
- `Application`: Main application instance
- `Window`: Platform window abstraction
- `Platform`: OS-specific implementations

## Data Flow

1. **User Input** → Platform Layer → Event System
2. **State Changes** → Entity Updates → View Re-rendering
3. **Render Tree** → Layout Engine → GPU Rendering
4. **Frame Output** → Platform Display

## Memory Model

- **Ownership**: Rust's ownership system ensures memory safety
- **Reference Counting**: `Rc<RefCell<T>>` for shared state
- **Weak References**: Prevent circular dependencies
- **Arena Allocation**: Efficient memory management for UI elements
