# Zed Performance Patterns - Implementation Guide for nwidgets

## Quick Reference: Pattern Implementations

### 1. SmallVec Implementation Examples

#### Before (Heap Allocation):
```rust
pub struct Widget {
    children: Vec<AnyElement>,
    event_handlers: Vec<EventHandler>,
}
```

#### After (Stack Allocation):
```rust
use smallvec::SmallVec;

pub struct Widget {
    // Most widgets have 1-4 children
    children: SmallVec<[AnyElement; 4]>,
    // Most widgets have 1-2 event handlers
    event_handlers: SmallVec<[EventHandler; 2]>,
}

// Usage:
impl Widget {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            event_handlers: SmallVec::new(),
        }
    }

    pub fn add_child(&mut self, child: AnyElement) {
        self.children.push(child);
    }

    pub fn add_handler(&mut self, handler: EventHandler) {
        self.event_handlers.push(handler);
    }
}
```

**Cargo.toml**:
```toml
[dependencies]
smallvec = "1.11"
```

---

### 2. Frame-Based Layout Cache

#### Implementation:
```rust
use std::sync::Arc;
use parking_lot::{Mutex, RwLock};
use collections::FxHashMap;

pub struct LayoutCache {
    previous_frame: Mutex<FrameCache>,
    current_frame: RwLock<FrameCache>,
}

#[derive(Default)]
struct FrameCache {
    layouts: FxHashMap<Arc<LayoutKey>, Arc<ComputedLayout>>,
    used_keys: Vec<Arc<LayoutKey>>,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct LayoutKey {
    widget_id: u64,
    available_width: u32,
    available_height: u32,
}

pub struct ComputedLayout {
    pub width: f32,
    pub height: f32,
    pub children_positions: Vec<(f32, f32)>,
}

impl LayoutCache {
    pub fn new() -> Self {
        Self {
            previous_frame: Mutex::new(FrameCache::default()),
            current_frame: RwLock::new(FrameCache::default()),
        }
    }

    pub fn get_or_compute<F>(
        &self,
        key: LayoutKey,
        compute: F,
    ) -> Arc<ComputedLayout>
    where
        F: FnOnce() -> ComputedLayout,
    {
        let current = self.current_frame.upgradable_read();
        
        // Check current frame cache
        if let Some(layout) = current.layouts.get(&Arc::new(key.clone())) {
            return layout.clone();
        }

        // Check previous frame cache
        let previous = self.previous_frame.lock();
        if let Some((key_arc, layout)) = previous.layouts.remove_entry(&Arc::new(key.clone())) {
            let mut current = RwLock::upgradable_read_guard::upgrade(current);
            current.layouts.insert(key_arc.clone(), layout.clone());
            current.used_keys.push(key_arc);
            return layout;
        }

        // Compute new layout
        drop(current);
        let layout = Arc::new(compute());
        let key_arc = Arc::new(key);
        
        let mut current = self.current_frame.write();
        current.layouts.insert(key_arc.clone(), layout.clone());
        current.used_keys.push(key_arc);
        
        layout
    }

    pub fn finish_frame(&self) {
        let mut prev = self.previous_frame.lock();
        let mut curr = self.current_frame.write();
        
        // Swap frames
        std::mem::swap(&mut *prev, &mut *curr);
        
        // Clear current frame
        curr.layouts.clear();
        curr.used_keys.clear();
    }
}
```

---

### 3. Debouncing Event Handler

#### Implementation:
```rust
use std::time::Duration;
use gpui::{Task, App, Context};

pub struct DebouncedHandler {
    pending_task: Option<Task<()>>,
    debounce_ms: u64,
}

impl DebouncedHandler {
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            pending_task: None,
            debounce_ms,
        }
    }

    pub fn handle_event<F>(&mut self, cx: &mut App, f: F)
    where
        F: FnOnce() + 'static,
    {
        // Cancel previous task
        self.pending_task = None;

        // Spawn new debounced task
        let debounce = Duration::from_millis(self.debounce_ms);
        self.pending_task = Some(cx.spawn(async move |_cx| {
            // Wait for debounce period
            // Note: In real implementation, use cx.background_executor().timer()
            tokio::time::sleep(debounce).await;
            f();
        }));
    }
}

// Usage:
pub struct TextInput {
    debounced_handler: DebouncedHandler,
}

impl TextInput {
    pub fn on_text_changed(&mut self, cx: &mut App, new_text: String) {
        self.debounced_handler.handle_event(cx, move || {
            println!("Text changed to: {}", new_text);
            // Perform expensive operation
        });
    }
}
```

---

### 4. FxHashMap for Style Lookups

#### Before:
```rust
use std::collections::HashMap;

pub struct StyleCache {
    styles: HashMap<StyleKey, StyleValue>,
}
```

#### After:
```rust
use rustc_hash::FxHashMap;

pub struct StyleCache {
    // FxHashMap is faster for small keys like TypeId
    styles: FxHashMap<StyleKey, StyleValue>,
}

impl StyleCache {
    pub fn new() -> Self {
        Self {
            styles: FxHashMap::default(),
        }
    }

    pub fn get(&self, key: &StyleKey) -> Option<&StyleValue> {
        self.styles.get(key)
    }

    pub fn insert(&mut self, key: StyleKey, value: StyleValue) {
        self.styles.insert(key, value);
    }
}
```

**Cargo.toml**:
```toml
[dependencies]
rustc-hash = "1.1"
```

---

### 5. Deferred Rendering for Overlays

#### Implementation:
```rust
use gpui::{Element, Window, App, Bounds, Pixels, AnyElement};

pub struct DeferredOverlay {
    child: Option<AnyElement>,
    priority: usize,
}

impl DeferredOverlay {
    pub fn new(child: AnyElement, priority: usize) -> Self {
        Self {
            child: Some(child),
            priority,
        }
    }
}

impl Element for DeferredOverlay {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let layout_id = self.child.as_mut().unwrap().request_layout(window, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        _cx: &mut App,
    ) {
        // Defer the drawing to later phase
        let child = self.child.take().unwrap();
        let element_offset = window.element_offset();
        window.defer_draw(child, element_offset, self.priority);
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
        // Paint is a no-op; actual painting happens in deferred phase
    }
}

// Usage:
pub fn render_tooltip(content: AnyElement) -> DeferredOverlay {
    DeferredOverlay::new(content, 100) // Higher priority = rendered on top
}
```

---

### 6. LRU Image Cache

#### Implementation:
```rust
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

pub struct ImageCacheItem {
    data: Arc<Vec<u8>>,
    size_bytes: usize,
}

pub struct LruImageCache {
    cache: HashMap<u64, ImageCacheItem>,
    usage_order: VecDeque<u64>,
    max_size_bytes: usize,
    current_size_bytes: usize,
}

impl LruImageCache {
    pub fn new(max_size_bytes: usize) -> Self {
        Self {
            cache: HashMap::new(),
            usage_order: VecDeque::new(),
            max_size_bytes,
            current_size_bytes: 0,
        }
    }

    pub fn get(&mut self, key: u64) -> Option<Arc<Vec<u8>>> {
        if let Some(item) = self.cache.get(&key) {
            // Move to end (most recently used)
            self.usage_order.retain(|&k| k != key);
            self.usage_order.push_back(key);
            return Some(item.data.clone());
        }
        None
    }

    pub fn insert(&mut self, key: u64, data: Arc<Vec<u8>>) {
        let size = data.len();

        // Remove old entries if necessary
        while self.current_size_bytes + size > self.max_size_bytes {
            if let Some(oldest_key) = self.usage_order.pop_front() {
                if let Some(item) = self.cache.remove(&oldest_key) {
                    self.current_size_bytes -= item.size_bytes;
                }
            }
        }

        // Insert new item
        self.current_size_bytes += size;
        self.cache.insert(key, ImageCacheItem {
            data,
            size_bytes: size,
        });
        self.usage_order.push_back(key);
    }

    pub fn clear(&mut self) {
        self.cache.clear();
        self.usage_order.clear();
        self.current_size_bytes = 0;
    }
}
```

---

### 7. Character Width Cache

#### Implementation:
```rust
use std::collections::HashMap;

pub struct CharWidthCache {
    // Fast path: ASCII characters (0-127)
    ascii_widths: [Option<f32>; 128],
    // Slow path: Extended characters
    extended_widths: HashMap<char, f32>,
}

impl CharWidthCache {
    pub fn new() -> Self {
        Self {
            ascii_widths: [None; 128],
            extended_widths: HashMap::new(),
        }
    }

    pub fn get_width<F>(&mut self, c: char, compute: F) -> f32
    where
        F: FnOnce() -> f32,
    {
        if (c as u32) < 128 {
            // ASCII fast path
            let idx = c as usize;
            if let Some(width) = self.ascii_widths[idx] {
                return width;
            }
            let width = compute();
            self.ascii_widths[idx] = Some(width);
            width
        } else {
            // Extended character slow path
            if let Some(&width) = self.extended_widths.get(&c) {
                return width;
            }
            let width = compute();
            self.extended_widths.insert(c, width);
            width
        }
    }

    pub fn clear(&mut self) {
        self.ascii_widths = [None; 128];
        self.extended_widths.clear();
    }
}
```

---

### 8. Bounds Tree for Hit Testing

#### Implementation:
```rust
use std::cmp;

const MAX_CHILDREN: usize = 12;

pub struct BoundsTree {
    nodes: Vec<Node>,
    root: Option<usize>,
    max_leaf: Option<usize>,
    insert_path: Vec<usize>,
    search_stack: Vec<usize>,
}

struct Node {
    bounds: Bounds,
    max_order: u32,
    kind: NodeKind,
}

enum NodeKind {
    Leaf { order: u32 },
    Internal { children: Vec<usize> },
}

pub struct Bounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl BoundsTree {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            root: None,
            max_leaf: None,
            insert_path: Vec::new(),
            search_stack: Vec::new(),
        }
    }

    pub fn insert(&mut self, bounds: Bounds, order: u32) -> usize {
        let node_idx = self.nodes.len();
        self.nodes.push(Node {
            bounds,
            max_order: order,
            kind: NodeKind::Leaf { order },
        });

        if self.root.is_none() {
            self.root = Some(node_idx);
            self.max_leaf = Some(node_idx);
        } else {
            // Insert into tree
            self.insert_path.clear();
            self.insert_into_tree(node_idx);
        }

        node_idx
    }

    fn insert_into_tree(&mut self, _new_node: usize) {
        // Implementation details for tree insertion
        // Similar to R-tree insertion algorithm
    }

    pub fn hit_test(&mut self, x: f32, y: f32) -> Option<usize> {
        self.search_stack.clear();
        
        if let Some(root) = self.root {
            self.search_stack.push(root);
        }

        let mut result = None;

        while let Some(node_idx) = self.search_stack.pop() {
            let node = &self.nodes[node_idx];
            
            if !self.point_in_bounds(x, y, &node.bounds) {
                continue;
            }

            match &node.kind {
                NodeKind::Leaf { .. } => {
                    result = Some(node_idx);
                }
                NodeKind::Internal { children } => {
                    for &child_idx in children {
                        self.search_stack.push(child_idx);
                    }
                }
            }
        }

        result
    }

    fn point_in_bounds(&self, x: f32, y: f32, bounds: &Bounds) -> bool {
        x >= bounds.x
            && x < bounds.x + bounds.width
            && y >= bounds.y
            && y < bounds.y + bounds.height
    }
}
```

---

## Performance Measurement

### Benchmarking Template

```rust
#[cfg(test)]
mod benches {
    use super::*;
    use criterion::{black_box, criterion_group, criterion_main, Criterion};

    fn bench_smallvec_vs_vec(c: &mut Criterion) {
        c.bench_function("vec_allocation", |b| {
            b.iter(|| {
                let mut v: Vec<i32> = Vec::new();
                for i in 0..10 {
                    v.push(black_box(i));
                }
            })
        });

        c.bench_function("smallvec_allocation", |b| {
            b.iter(|| {
                let mut v: SmallVec<[i32; 16]> = SmallVec::new();
                for i in 0..10 {
                    v.push(black_box(i));
                }
            })
        });
    }

    fn bench_layout_cache(c: &mut Criterion) {
        let cache = LayoutCache::new();
        
        c.bench_function("layout_cache_hit", |b| {
            b.iter(|| {
                let key = LayoutKey {
                    widget_id: 1,
                    available_width: 800,
                    available_height: 600,
                };
                let _ = cache.get_or_compute(key, || {
                    ComputedLayout {
                        width: 100.0,
                        height: 50.0,
                        children_positions: vec![],
                    }
                });
            })
        });
    }

    criterion_group!(benches, bench_smallvec_vs_vec, bench_layout_cache);
    criterion_main!(benches);
}
```

---

## Migration Checklist

- [ ] Add dependencies: `smallvec`, `rustc-hash`
- [ ] Identify hot paths (profiling)
- [ ] Replace `Vec<T>` with `SmallVec<[T; N]>` in:
  - [ ] Widget children collections
  - [ ] Event handler lists
  - [ ] Style property collections
- [ ] Implement frame-based layout cache
- [ ] Add debouncing to event handlers
- [ ] Replace `HashMap` with `FxHashMap` for:
  - [ ] Style lookups
  - [ ] TypeId-based caches
- [ ] Implement deferred rendering for overlays
- [ ] Add image/asset caching
- [ ] Profile and measure improvements
- [ ] Document performance characteristics
- [ ] Add benchmarks

---

## Performance Targets

| Optimization | Expected Improvement | Effort |
|---|---|---|
| SmallVec | 10-20% memory | Low |
| Debouncing | 30-50% event processing | Low |
| Layout cache | 20-30% layout time | Medium |
| FxHashMap | 15-25% lookup speed | Low |
| Deferred rendering | 10-15% render time | Medium |
| Image cache | 20-40% (asset-heavy) | Medium |
| Bounds tree | 5-10% (complex UIs) | High |
| Char width cache | 10-20% (text-heavy) | Low |

