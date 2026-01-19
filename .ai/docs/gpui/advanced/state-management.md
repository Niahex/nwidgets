# State Management

Advanced patterns for managing application state in GPUI.

## Entity-Based State

### Basic Entity Pattern
```rust
struct AppData {
    items: Vec<String>,
    selected_index: Option<usize>,
}

impl AppData {
    fn add_item(&mut self, item: String, cx: &mut Context<Self>) {
        self.items.push(item);
        cx.notify();
    }
    
    fn select_item(&mut self, index: usize, cx: &mut Context<Self>) {
        self.selected_index = Some(index);
        cx.notify();
    }
}
```

### Shared State Between Views
```rust
struct ViewA {
    data: Model<AppData>,
}

struct ViewB {
    data: Model<AppData>, // Same data reference
}

// Both views automatically update when data changes
```

## Global State

### Global State Pattern
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

## Event-Driven Updates

### Subscription Pattern
```rust
struct DataStore {
    data: Vec<String>,
}

impl EventEmitter<DataStoreEvent> for DataStore {}

#[derive(Clone)]
enum DataStoreEvent {
    ItemAdded(String),
    ItemRemoved(usize),
}

// Subscribe to events
let subscription = cx.subscribe(&data_store, |subscriber, event, cx| {
    match event {
        DataStoreEvent::ItemAdded(item) => {
            // Handle new item
        }
        DataStoreEvent::ItemRemoved(index) => {
            // Handle removed item
        }
    }
});
```

## Best Practices

1. **Keep State Local**: Use entities for component-specific state
2. **Global Sparingly**: Only for truly global settings
3. **Immutable Updates**: Prefer replacing over mutating
4. **Event Sourcing**: Use events for complex state changes
5. **Weak References**: Avoid circular dependencies
