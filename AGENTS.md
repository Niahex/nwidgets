# nwidgets Coding Guidelines

## Rust Coding Standards

* Prioritize code correctness and clarity. Speed and efficiency are secondary priorities unless otherwise specified.
* Do not write organizational comments that summarize the code. Comments should only explain "why" the code is written in a specific way when the reason is tricky or non-obvious.
* Prefer implementing functionality in existing files unless it is a new logical component. Avoid creating many small files.
* Avoid using functions that panic like `unwrap()`. Instead, use mechanisms like `?` to propagate errors.
* Be careful with operations like indexing which may panic if the indexes are out of bounds.
* Never silently discard errors with `let _ =` on fallible operations. Always handle errors appropriately:
  - Propagate errors with `?` when the calling function should handle them
  - Use `.log_err()` or similar when you need to ignore errors but want visibility
  - Use explicit error handling with `match` or `if let Err(...)` when you need custom logic
  - Example: avoid `let _ = client.request(...).await?;` - use `client.request(...).await?;` instead
* When implementing async operations that may fail, ensure errors propagate to the UI layer so users get meaningful feedback.
* **Never create files with `mod.rs` paths** - prefer `src/some_module.rs` instead of `src/some_module/mod.rs`.
* When creating new modules, use descriptive file names that match the module content (e.g., `audio_service.rs` instead of `mod.rs`).
* Avoid creative additions unless explicitly requested.
* Use full words for variable names (no abbreviations like "q" for "queue", "ws" for "workspace", "cx" is acceptable as it's a GPUI convention).
* Use variable shadowing to scope clones in async contexts for clarity, minimizing the lifetime of borrowed references.
  Example:
  ```rust
  cx.spawn({
      let task_ran = task_ran.clone();
      async move {
          *task_ran.borrow_mut() = true;
      }
  });
  ```

## GPUI Framework

GPUI is a UI framework which also provides primitives for state and concurrency management. nwidgets uses a custom fork with Wayland support.

### Context

Context types allow interaction with global state, windows, entities, and system services. They are typically passed to functions as the argument named `cx`. When a function takes callbacks they come after the `cx` parameter.

* `App` is the root context type, providing access to global state and read and update of entities.
* `Context<T>` is provided when updating an `Entity<T>`. This context dereferences into `App`, so functions which take `&App` can also take `&Context<T>`.
* `AsyncApp` and `AsyncWindowContext` are provided by `cx.spawn` and `cx.spawn_in`. These can be held across await points.

### Window

`Window` provides access to the state of an application window. It is passed to functions as an argument named `window` and comes before `cx` when present. It is used for managing focus, dispatching actions, directly drawing, getting user input state, etc.

### Entities

An `Entity<T>` is a handle to state of type `T`. With `thing: Entity<T>`:

* `thing.entity_id()` returns `EntityId`
* `thing.downgrade()` returns `WeakEntity<T>`
* `thing.read(cx: &App)` returns `&T`.
* `thing.read_with(cx, |thing: &T, cx: &App| ...)` returns the closure's return value.
* `thing.update(cx, |thing: &mut T, cx: &mut Context<T>| ...)` allows the closure to mutate the state, and provides a `Context<T>` for interacting with the entity. It returns the closure's return value.
* `thing.update_in(cx, |thing: &mut T, window: &mut Window, cx: &mut Context<T>| ...)` takes a `AsyncWindowContext` or `VisualTestContext`. It's the same as `update` while also providing the `Window`.

Within the closures, the inner `cx` provided to the closure must be used instead of the outer `cx` to avoid issues with multiple borrows.

Trying to update an entity while it's already being updated must be avoided as this will cause a panic.

When `read_with`, `update`, or `update_in` are used with an async context, the closure's return value is wrapped in an `anyhow::Result`.

`WeakEntity<T>` is a weak handle. It has `read_with`, `update`, and `update_in` methods that work the same, but always return an `anyhow::Result` so that they can fail if the entity no longer exists. This can be useful to avoid memory leaks - if entities have mutually recursive handles to each other they will never be dropped.

### Concurrency

All use of entities and UI rendering occurs on a single foreground thread.

`cx.spawn(async move |cx| ...)` runs an async closure on the foreground thread. Within the closure, `cx` is `&mut AsyncApp`.

When the outer cx is a `Context<T>`, the use of `spawn` instead looks like `cx.spawn(async move |this, cx| ...)`, where `this: WeakEntity<T>` and `cx: &mut AsyncApp`.

To do work on other threads, use `gpui_tokio::Tokio::spawn(cx, async move { ... })` for Tokio runtime tasks. Often this background task is awaited on by a foreground task which uses the results to update state.

Both `cx.spawn` and `gpui_tokio::Tokio::spawn` return a `Task<R>`, which is a future that can be awaited upon. If this task is dropped, then its work is cancelled. To prevent this one of the following must be done:

* Awaiting the task in some other async context.
* Detaching the task via `task.detach()` or `task.detach_and_log_err(cx)`, allowing it to run indefinitely.
* Storing the task in a field, if the work should be halted when the struct is dropped.

A task which doesn't do anything but provide a value can be created with `Task::ready(value)`.

### Elements

The `Render` trait is used to render some state into an element tree that is laid out using flexbox layout. An `Entity<T>` where `T` implements `Render` is sometimes called a "view".

Example:

```rust
struct TextWithBorder(SharedString);

impl Render for TextWithBorder {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().border_1().child(self.0.clone())
    }
}
```

Since `impl IntoElement for SharedString` exists, it can be used as an argument to `child`. `SharedString` is used to avoid copying strings, and is either an `&'static str` or `Arc<str>`.

UI components that are constructed just to be turned into elements can instead implement the `RenderOnce` trait, which is similar to `Render`, but its `render` method takes ownership of `self` and receives `&mut App` instead of `&mut Context<Self>`. Types that implement this trait can use `#[derive(IntoElement)]` to use them directly as children.

The style methods on elements are similar to those used by Tailwind CSS.

If some attributes or children of an element tree are conditional, `.when(condition, |this| ...)` can be used to run the closure only when `condition` is true. Similarly, `.when_some(option, |this, value| ...)` runs the closure when the `Option` has a value.

### Input Events

Input event handlers can be registered on an element via methods like `.on_click(|event, window, cx: &mut App| ...)`.

Often event handlers will want to update the entity that's in the current `Context<T>`. The `cx.listener` method provides this - its use looks like `.on_click(cx.listener(|this: &mut T, event, window, cx: &mut Context<T>| ...)`.

### Actions

Actions are dispatched via user keyboard interaction or in code via `window.dispatch_action(SomeAction.boxed_clone(), cx)` or `focus_handle.dispatch_action(&SomeAction, window, cx)`.

Actions with no data defined with the `actions!(some_namespace, [SomeAction, AnotherAction])` macro call. Otherwise the `Action` derive macro is used. Doc comments on actions are displayed to the user.

Action handlers can be registered on an element via the event handler `.on_action(|action, window, cx| ...)`. Like other event handlers, this is often used with `cx.listener`.

### Notify

When a view's state has changed in a way that may affect its rendering, it should call `cx.notify()`. This will cause the view to be rerendered. It will also cause any observe callbacks registered for the entity with `cx.observe` to be called.

### Entity Events

While updating an entity (`cx: Context<T>`), it can emit an event using `cx.emit(event)`. Entities register which events they can emit by declaring `impl EventEmitter<EventType> for EntityType {}`.

Other entities can then register a callback to handle these events by doing `cx.subscribe(other_entity, |this, other_entity, event, cx| ...)`. This will return a `Subscription` which deregisters the callback when dropped. Typically `cx.subscribe` happens when creating a new entity and the subscriptions are stored in a `_subscriptions: Vec<Subscription>` field.

## nwidgets-Specific Guidelines

### Service Architecture

Services in nwidgets follow an event-driven architecture with global singletons:

* All services use `Arc<RwLock<T>>` for shared state
* Services communicate via `futures::channel::mpsc::unbounded` channels
* Worker tasks run in Tokio runtime via `gpui_tokio::Tokio::spawn`
* UI updates happen in GPUI executor via `cx.spawn`
* Always compare state before emitting events to avoid unnecessary re-renders

Example service pattern:

```rust
pub struct MyService {
    state: Arc<RwLock<MyState>>,
}

impl EventEmitter<StateChanged> for MyService {}

impl MyService {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let state = Arc::new(RwLock::new(MyState::default()));
        let (ui_tx, mut ui_rx) = futures::channel::mpsc::unbounded();

        // Worker task (Tokio)
        gpui_tokio::Tokio::spawn(cx, async move {
            Self::worker(ui_tx).await
        }).detach();

        // UI task (GPUI)
        let state_clone = Arc::clone(&state);
        cx.spawn(move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
            let mut cx = cx.clone();
            async move {
                while let Some(update) = ui_rx.next().await {
                    let changed = {
                        let mut current = state_clone.write();
                        if *current != update {
                            *current = update;
                            true
                        } else {
                            false
                        }
                    };
                    if changed {
                        let _ = this.update(&mut cx, |_, cx| {
                            cx.emit(StateChanged);
                            cx.notify();
                        });
                    }
                }
            }
        }).detach();

        Self { state }
    }

    async fn worker(tx: UnboundedSender<MyState>) {
        // Event-driven worker logic
    }
}

// Global singleton pattern
struct GlobalMyService(Entity<MyService>);
impl Global for GlobalMyService {}

impl MyService {
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalMyService>().0.clone()
    }

    pub fn init(cx: &mut App) -> Entity<Self> {
        let service = cx.new(Self::new);
        cx.set_global(GlobalMyService(service.clone()));
        service
    }
}
```

### Performance Guidelines

* **Event-driven over polling**: Use `tokio::sync::Notify` or event streams instead of polling loops
* **State comparison**: Always compare state before emitting events to avoid unnecessary re-renders
* **Deferred rendering**: Use `deferred()` for complex views that are not always visible
* **Lazy loading**: Limit lists with `.take(N)` to avoid rendering hundreds of items
* **SharedString caching**: Use `SharedString` for UI text that changes rarely
* **On-demand monitoring**: Pause monitoring services when not needed

See `.ai/performance-guide.md` for detailed performance patterns.

### Widget Structure

Widgets follow a modular structure:

```
widgets/
├── my_widget/
│   ├── mod.rs              # Re-exports only
│   ├── types.rs            # Type definitions
│   ├── service/
│   │   └── my_service.rs   # Service logic
│   ├── widget/
│   │   └── my_widget.rs    # UI rendering
│   └── window/
│       └── window_manager.rs  # Window management
```

### Wayland Integration

* Use layer shell for panels and overlays
* Handle fullscreen state via Hyprland events
* Respect compositor keyboard focus
* Use proper anchoring for positioned windows

### CEF Integration

* Initialize CEF subprocess before GPUI
* Disable GPU rendering for stability
* Handle clipboard injection carefully
* Use message handlers for JS communication

## Build Guidelines

* Use `cargo build --release` for production builds
* Profile with `perf` or `flamegraph` for performance issues
* Test on both Nvidia and AMD GPUs
* Verify Wayland compatibility

## Testing

* Test services independently before integration
* Verify event-driven behavior (no polling)
* Check CPU usage in idle state (<1%)
* Ensure frame time stays under 16ms (60 FPS)
