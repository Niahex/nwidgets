# Context API

## Context<T>

The context for entity operations and UI building.

```rust
impl<T> Context<T> {
    pub fn new<U>(&mut self, build: impl FnOnce(&mut Context<U>) -> U) -> Model<U>
    pub fn update<U, R>(&mut self, model: &Model<U>, update: impl FnOnce(&mut U, &mut Context<U>) -> R) -> R
    pub fn read<U>(&self, model: &Model<U>) -> &U
    pub fn notify(&mut self)
    pub fn listener<E>(&mut self, f: impl Fn(&mut T, &E, &mut Context<T>) + 'static) -> impl Fn(&E, &mut Context<T>)
    pub fn spawn<F>(&mut self, f: F) -> Task<F::Output> where F: Future + 'static
    pub fn defer(&mut self, f: impl FnOnce(&mut T, &mut Context<T>) + 'static)
}
```

### Entity Management

#### `new`
Creates a new entity.

```rust
let counter = cx.new(|_| Counter { value: 0 });
```

#### `update`
Updates an entity's state.

```rust
counter.update(cx, |counter, cx| {
    counter.value += 1;
    cx.notify(); // Trigger re-render
});
```

#### `read`
Reads an entity's state (immutable).

```rust
let value = counter.read(cx).value;
```

### Event Handling

#### `listener`
Creates an event listener for the current view.

```rust
.on_click(cx.listener(|this, event, cx| {
    // Handle click event
}))
```

#### `notify`
Triggers a re-render of the current view.

```rust
cx.notify();
```

### Async Operations

#### `spawn`
Spawns an async task.

```rust
cx.spawn(|this, mut cx| async move {
    let data = fetch_data().await;
    this.update(&mut cx, |this, cx| {
        this.data = Some(data);
        cx.notify();
    }).ok();
})
```

#### `defer`
Defers execution to the next frame.

```rust
cx.defer(|this, cx| {
    this.process_updates(cx);
});
```

## Model<T>

A smart pointer to an entity managed by GPUI.

```rust
impl<T> Model<T> {
    pub fn read<'a>(&self, cx: &'a Context<impl Any>) -> &'a T
    pub fn update<R>(&self, cx: &mut Context<impl Any>, f: impl FnOnce(&mut T, &mut Context<T>) -> R) -> R
    pub fn entity_id(&self) -> EntityId
}
```

### Example Usage

```rust
struct Counter {
    value: i32,
}

impl Counter {
    fn increment(&mut self, cx: &mut Context<Self>) {
        self.value += 1;
        cx.notify();
    }
}

// Create model
let counter = cx.new(|_| Counter { value: 0 });

// Read state
let current_value = counter.read(cx).value;

// Update state
counter.update(cx, |counter, cx| {
    counter.increment(cx);
});
```

## Global State

### Global Trait

```rust
pub trait Global: 'static {}

impl<T: 'static> Global for T {}
```

### Global Operations

```rust
impl Context<T> {
    pub fn global<G: Global>(&self) -> &G
    pub fn update_global<G: Global, R>(&mut self, f: impl FnOnce(&mut G, &mut Context<T>) -> R) -> R
    pub fn set_global<G: Global>(&mut self, global: G)
}
```

### Example

```rust
#[derive(Clone)]
struct AppSettings {
    theme: Theme,
    font_size: f32,
}

impl Global for AppSettings {}

// Set global state
cx.set_global(AppSettings {
    theme: Theme::Dark,
    font_size: 14.0,
});

// Read global state
let settings = cx.global::<AppSettings>();

// Update global state
cx.update_global::<AppSettings, _>(|settings, cx| {
    settings.font_size = 16.0;
});
```
