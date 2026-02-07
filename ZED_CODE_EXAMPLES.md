# Zed Performance Patterns - Code Examples and Implementation Guide

## 1. MEMORY OPTIMIZATION EXAMPLES

### 1.1 SharedString Implementation Pattern

**Zed's Implementation:**
```rust
// From gpui/src/shared_string.rs
use derive_more::{Deref, DerefMut};
use util::arc_cow::ArcCow;

#[derive(Deref, DerefMut, Eq, PartialEq, PartialOrd, Ord, Hash, Clone)]
pub struct SharedString(ArcCow<'static, str>);

impl SharedString {
    pub const fn new_static(str: &'static str) -> Self {
        Self(ArcCow::Borrowed(str))
    }

    pub fn new(str: impl Into<Arc<str>>) -> Self {
        SharedString(ArcCow::Owned(str.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for SharedString {
    fn from(val: String) -> Self {
        SharedString::new(val)
    }
}

impl From<&str> for SharedString {
    fn from(val: &str) -> Self {
        SharedString::new(val.to_string())
    }
}
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/shared_string.rs
use std::sync::Arc;
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut, Eq, PartialEq, Hash, Clone)]
pub struct SharedString(ArcCow<'static, str>);

// Usage in components
pub struct Button {
    label: SharedString,  // Cheap to clone
    on_click: Option<Box<dyn Fn()>>,
}

impl Button {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            on_click: None,
        }
    }
}

// Static strings (zero-cost)
const BUTTON_OK: SharedString = SharedString::new_static("OK");
const BUTTON_CANCEL: SharedString = SharedString::new_static("Cancel");

// Usage
let btn = Button::new("Click me");  // Owned string
let btn2 = Button::new(BUTTON_OK);  // Static string
```

### 1.2 SmallVec Usage Pattern

**Zed's Implementation:**
```rust
// From gpui/src/window.rs
use smallvec::SmallVec;

pub struct Window {
    // Focus path tracking
    pub(crate) previous_focus_path: SmallVec<[FocusId; 8]>,
    pub(crate) current_focus_path: SmallVec<[FocusId; 8]>,
    
    // Element ID stack
    pub(crate) element_id_stack: SmallVec<[ElementId; 32]>,
    
    // Keystroke buffer
    pub(crate) keystrokes: SmallVec<[Keystroke; 1]>,
    
    // Deferred draws
    pub(crate) deferred_draws: Vec<DeferredDraw>,
}

// Usage in rendering
let mut sorted_deferred_draws =
    (0..self.next_frame.deferred_draws.len()).collect::<SmallVec<[_; 8]>>();
sorted_deferred_draws.sort_by_key(|ix| self.next_frame.deferred_draws[*ix].priority);
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/element.rs
use smallvec::SmallVec;

pub struct ElementContext {
    // Typical nesting depth is 8-16 levels
    element_stack: SmallVec<[ElementId; 32]>,
    
    // Focus path (typical depth 4-8)
    focus_path: SmallVec<[FocusId; 8]>,
    
    // Event listeners (typical 2-4 per element)
    listeners: SmallVec<[EventListener; 4]>,
    
    // Deferred renders (typical 4-8)
    deferred: SmallVec<[DeferredRender; 8]>,
}

impl ElementContext {
    pub fn push_element(&mut self, id: ElementId) {
        self.element_stack.push(id);
    }
    
    pub fn pop_element(&mut self) {
        self.element_stack.pop();
    }
    
    pub fn add_listener(&mut self, listener: EventListener) {
        self.listeners.push(listener);
    }
}

// Profiling to determine optimal sizes
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn profile_element_stack_depth() {
        // Measure typical element tree depth
        // Adjust SmallVec capacity based on results
    }
}
```

### 1.3 Rc vs Arc Pattern

**Zed's Implementation:**
```rust
// From gpui/src/window.rs
use std::rc::Rc;
use std::sync::Arc;

pub struct Window {
    // Single-threaded UI state - use Rc
    pub(crate) active: Rc<Cell<bool>>,
    pub(crate) hovered: Rc<Cell<bool>>,
    pub(crate) needs_present: Rc<Cell<bool>>,
    
    // Mutable single-threaded state - use Rc<RefCell<>>
    pub(crate) next_frame_callbacks: Rc<RefCell<Vec<FrameCallback>>>,
    
    // Cross-thread resources - use Arc
    pub(crate) sprite_atlas: Arc<dyn PlatformAtlas>,
    pub(crate) text_system: Arc<WindowTextSystem>,
}

// From elements/list.rs
pub struct ListState(Rc<RefCell<StateInner>>);

impl ListState {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(StateInner {
            items: SumTree::new(),
            // ...
        })))
    }
    
    pub fn clone(&self) -> Self {
        Self(self.0.clone())  // Cheap Rc clone
    }
}
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/state.rs
use std::rc::Rc;
use std::cell::{Cell, RefCell};

pub struct ComponentState {
    // Copy types - use Cell (no RefCell overhead)
    is_hovered: Rc<Cell<bool>>,
    is_focused: Rc<Cell<bool>>,
    is_active: Rc<Cell<bool>>,
    
    // Mutable state - use RefCell
    data: Rc<RefCell<ComponentData>>,
}

pub struct ComponentData {
    value: String,
    metadata: HashMap<String, String>,
}

impl ComponentState {
    pub fn new() -> Self {
        Self {
            is_hovered: Rc::new(Cell::new(false)),
            is_focused: Rc::new(Cell::new(false)),
            is_active: Rc::new(Cell::new(false)),
            data: Rc::new(RefCell::new(ComponentData {
                value: String::new(),
                metadata: HashMap::new(),
            })),
        }
    }
    
    pub fn set_hovered(&self, hovered: bool) {
        self.is_hovered.set(hovered);  // No RefCell borrow
    }
    
    pub fn update_data(&self, f: impl FnOnce(&mut ComponentData)) {
        f(&mut self.data.borrow_mut());  // Single borrow
    }
}
```

### 1.4 OnceLock for Lazy Initialization

**Zed's Implementation:**
```rust
// From platform/windows/util.rs
use std::sync::OnceLock;

static ARROW: OnceLock<SafeCursor> = OnceLock::new();
static IBEAM: OnceLock<SafeCursor> = OnceLock::new();
static CROSS: OnceLock<SafeCursor> = OnceLock::new();

pub fn arrow_cursor() -> &'static SafeCursor {
    ARROW.get_or_init(|| SafeCursor::new(IDC_ARROW))
}

pub fn ibeam_cursor() -> &'static SafeCursor {
    IBEAM.get_or_init(|| SafeCursor::new(IDC_IBEAM))
}
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/theme.rs
use std::sync::OnceLock;

pub struct Theme {
    colors: HashMap<String, Color>,
    fonts: HashMap<String, Font>,
}

static DEFAULT_THEME: OnceLock<Theme> = OnceLock::new();
static DARK_THEME: OnceLock<Theme> = OnceLock::new();

pub fn default_theme() -> &'static Theme {
    DEFAULT_THEME.get_or_init(|| {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), Color::rgb(0, 120, 215));
        colors.insert("secondary".to_string(), Color::rgb(100, 100, 100));
        
        Theme {
            colors,
            fonts: HashMap::new(),
        }
    })
}

pub fn dark_theme() -> &'static Theme {
    DARK_THEME.get_or_init(|| {
        // Initialize dark theme
        Theme {
            colors: HashMap::new(),
            fonts: HashMap::new(),
        }
    })
}

// Usage - no allocation after first call
let theme = default_theme();
let primary_color = theme.colors.get("primary");
```

---

## 2. GPUI-SPECIFIC OPTIMIZATION EXAMPLES

### 2.1 Deferred Rendering Pattern

**Zed's Implementation:**
```rust
// From elements/deferred.rs
pub fn deferred(child: impl IntoElement) -> Deferred {
    Deferred {
        child: Some(child.into_any_element()),
        priority: 0,
    }
}

pub struct Deferred {
    child: Option<AnyElement>,
    priority: usize,
}

impl Element for Deferred {
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
        let child = self.child.take().unwrap();
        let element_offset = window.element_offset();
        window.defer_draw(child, element_offset, self.priority)
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
        // Empty - painting happens later
    }
}

impl Deferred {
    pub fn priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/elements/deferred.rs
use crate::{Element, AnyElement, Window, App};

pub fn deferred(child: impl IntoElement) -> Deferred {
    Deferred {
        child: Some(child.into_any_element()),
        priority: 0,
    }
}

pub struct Deferred {
    child: Option<AnyElement>,
    priority: usize,
}

impl Element for Deferred {
    fn layout(&mut self, window: &mut Window, cx: &mut App) -> LayoutId {
        self.child.as_mut().unwrap().layout(window, cx)
    }

    fn paint(&mut self, window: &mut Window, cx: &mut App) {
        // Defer painting to later in frame
        if let Some(child) = self.child.take() {
            window.defer_paint(child, self.priority);
        }
    }
}

// Usage in components
pub fn tooltip(content: impl IntoElement) -> impl IntoElement {
    deferred(content).priority(100)  // High priority (on top)
}

pub fn overlay(content: impl IntoElement) -> impl IntoElement {
    deferred(content).priority(50)   // Medium priority
}
```

### 2.2 Event Handling with Capture/Bubble

**Zed's Implementation:**
```rust
// From div.rs
pub enum DispatchPhase {
    Capture,
    Bubble,
}

pub fn on_mouse_down(
    &mut self,
    button: MouseButton,
    listener: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) {
    self.mouse_down_listeners
        .push(Box::new(move |event, phase, hitbox, window, cx| {
            if phase == DispatchPhase::Bubble
                && event.button == button
                && hitbox.is_hovered(window)
            {
                (listener)(event, window, cx)
            }
        }));
}

pub fn capture_any_mouse_down(
    &mut self,
    listener: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
) {
    self.mouse_down_listeners
        .push(Box::new(move |event, phase, hitbox, window, cx| {
            if phase == DispatchPhase::Capture && hitbox.is_hovered(window) {
                (listener)(event, window, cx)
            }
        }));
}
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/event.rs
use smallvec::SmallVec;

pub enum DispatchPhase {
    Capture,
    Bubble,
}

pub struct EventDispatcher {
    listeners: SmallVec<[EventListener; 4]>,
}

pub struct EventListener {
    phase: DispatchPhase,
    handler: Box<dyn Fn(&Event) -> bool>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            listeners: SmallVec::new(),
        }
    }

    pub fn on_capture(&mut self, handler: impl Fn(&Event) -> bool + 'static) {
        self.listeners.push(EventListener {
            phase: DispatchPhase::Capture,
            handler: Box::new(handler),
        });
    }

    pub fn on_bubble(&mut self, handler: impl Fn(&Event) -> bool + 'static) {
        self.listeners.push(EventListener {
            phase: DispatchPhase::Bubble,
            handler: Box::new(handler),
        });
    }

    pub fn dispatch(&self, event: &Event, phase: DispatchPhase) -> bool {
        for listener in &self.listeners {
            if listener.phase == phase {
                if (listener.handler)(event) {
                    return true;  // Event handled
                }
            }
        }
        false
    }
}

// Usage
let mut dispatcher = EventDispatcher::new();

// Capture phase - intercept before children
dispatcher.on_capture(|event| {
    if let Event::MouseDown(e) = event {
        println!("Capturing mouse down at {:?}", e.position);
        false  // Don't consume
    } else {
        false
    }
});

// Bubble phase - handle after children
dispatcher.on_bubble(|event| {
    if let Event::MouseDown(e) = event {
        println!("Bubbling mouse down at {:?}", e.position);
        true  // Consume event
    } else {
        false
    }
});
```

### 2.3 Subscription System

**Zed's Implementation:**
```rust
// From app.rs
pub fn observe<W: 'static>(
    &mut self,
    entity: &Entity<W>,
    mut on_notify: impl FnMut(Entity<W>, &mut App) + 'static,
) -> Subscription {
    let entity_id = entity.id();
    let handler = Box::new(move |_: &mut App| {
        on_notify(Entity::from_id(entity_id), cx)
    });
    
    self.new_observer(entity_id, handler)
}

pub fn new_observer(&mut self, key: EntityId, value: Handler) -> Subscription {
    let subscription = self.observers.insert(key, value);
    Subscription::new(subscription)
}
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/subscription.rs
use std::rc::Rc;
use std::cell::RefCell;

pub struct Subscription {
    inner: Rc<RefCell<SubscriptionInner>>,
}

pub struct SubscriptionInner {
    handler: Option<Box<dyn Fn()>>,
    active: bool,
}

impl Subscription {
    pub fn new(handler: Box<dyn Fn()>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(SubscriptionInner {
                handler: Some(handler),
                active: true,
            })),
        }
    }

    pub fn unsubscribe(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.active = false;
        inner.handler = None;
    }

    pub fn notify(&self) {
        let inner = self.inner.borrow();
        if inner.active {
            if let Some(handler) = &inner.handler {
                handler();
            }
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.unsubscribe();
    }
}

// Usage
pub struct State {
    value: i32,
    subscribers: Vec<Subscription>,
}

impl State {
    pub fn subscribe(&mut self, handler: Box<dyn Fn()>) -> Subscription {
        let subscription = Subscription::new(handler);
        self.subscribers.push(subscription.clone());
        subscription
    }

    pub fn set_value(&mut self, value: i32) {
        self.value = value;
        for sub in &self.subscribers {
            sub.notify();
        }
    }
}
```

---

## 3. ASYNC/CONCURRENCY EXAMPLES

### 3.1 Task Spawning Pattern

**Zed's Implementation:**
```rust
// From executor.rs
pub struct BackgroundExecutor {
    inner: scheduler::BackgroundExecutor,
    dispatcher: Arc<dyn PlatformDispatcher>,
}

pub struct ForegroundExecutor {
    inner: scheduler::ForegroundExecutor,
    dispatcher: Arc<dyn PlatformDispatcher>,
    not_send: PhantomData<Rc<()>>,
}

pub fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) -> Task<R>
where
    R: Send + 'static,
{
    self.spawn_with_priority(Priority::default(), future.boxed())
}

pub fn detach_and_log_err(self, cx: &App) {
    let location = core::panic::Location::caller();
    cx.foreground_executor()
        .spawn(self.log_tracked_err(*location))
        .detach();
}
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/executor.rs
use std::rc::Rc;
use std::marker::PhantomData;
use std::future::Future;
use std::pin::Pin;

pub struct BackgroundExecutor {
    // Implementation
}

pub struct ForegroundExecutor {
    // PhantomData prevents Send/Sync
    _not_send: PhantomData<Rc<()>>,
}

pub struct Task<T> {
    future: Pin<Box<dyn Future<Output = T>>>,
}

impl<T> Task<T> {
    pub fn ready(val: T) -> Self {
        Self {
            future: Box::pin(async move { val }),
        }
    }

    pub fn is_ready(&self) -> bool {
        // Check if future is ready
        false
    }

    pub fn detach(self) {
        // Run to completion in background
    }
}

impl BackgroundExecutor {
    pub fn spawn<R>(&self, future: impl Future<Output = R> + Send + 'static) -> Task<R>
    where
        R: Send + 'static,
    {
        Task {
            future: Box::pin(future),
        }
    }
}

impl ForegroundExecutor {
    pub fn spawn<R>(&self, future: impl Future<Output = R> + 'static) -> Task<R>
    where
        R: 'static,
    {
        Task {
            future: Box::pin(future),
        }
    }
}

// Usage
async fn load_data() -> String {
    // Simulate async work
    "data".to_string()
}

let bg_executor = BackgroundExecutor::new();
let task = bg_executor.spawn(load_data());
// task.await to get result
```

### 3.2 Oneshot Channel Pattern

**Zed's Implementation:**
```rust
// From platform.rs
use futures::channel::oneshot;

let (sources_tx, sources_rx) = oneshot::channel();

// Send result
sources_tx.send(result).ok();

// Receive result
let result = sources_rx.await;
```

**How to Apply to nwidgets:**
```rust
// In nwidgets/src/dialog.rs
use futures::channel::oneshot;

pub struct FileDialog {
    title: String,
}

impl FileDialog {
    pub async fn show(&self) -> Option<String> {
        let (tx, rx) = oneshot::channel();
        
        // Show dialog and send result
        self.show_native(tx);
        
        // Wait for result
        rx.await.ok()
    }

    fn show_native(&self, tx: oneshot::Sender<String>) {
        // Platform-specific dialog code
        // When user selects file:
        let _ = tx.send(selected_path);
    }
}

// Usage
let dialog = FileDialog {
    title: "Open File".to_string(),
};

if let Some(path) = dialog.show().await {
    println!("Selected: {}", path);
}
```

---

## 4. PERFORMANCE PROFILING EXAMPLES

### 4.1 Measuring Allocation Patterns

```rust
// In nwidgets/benches/allocations.rs
#[bench]
fn bench_smallvec_vs_vec(b: &mut Bencher) {
    b.iter(|| {
        // SmallVec - typically faster for small collections
        let mut sv: SmallVec<[i32; 8]> = SmallVec::new();
        for i in 0..5 {
            sv.push(i);
        }
        black_box(sv)
    });
}

#[bench]
fn bench_shared_string_clone(b: &mut Bencher) {
    let s = SharedString::new("Hello, World!");
    b.iter(|| {
        let cloned = s.clone();  // Should be cheap (Arc clone)
        black_box(cloned)
    });
}

#[bench]
fn bench_rc_vs_arc(b: &mut Bencher) {
    let rc = Rc::new(vec![1, 2, 3, 4, 5]);
    b.iter(|| {
        let cloned = rc.clone();  // Faster than Arc
        black_box(cloned)
    });
}
```

### 4.2 Frame Time Profiling

```rust
// In nwidgets/src/profiler.rs
pub struct FrameProfiler {
    frame_times: Vec<Duration>,
    paint_times: Vec<Duration>,
    layout_times: Vec<Duration>,
}

impl FrameProfiler {
    pub fn profile_frame<F>(&mut self, f: F) -> Duration
    where
        F: FnOnce(),
    {
        let start = Instant::now();
        f();
        let duration = start.elapsed();
        self.frame_times.push(duration);
        duration
    }

    pub fn average_frame_time(&self) -> Duration {
        let sum: Duration = self.frame_times.iter().sum();
        sum / self.frame_times.len() as u32
    }

    pub fn report(&self) {
        let avg = self.average_frame_time();
        let max = self.frame_times.iter().max().copied().unwrap_or_default();
        let min = self.frame_times.iter().min().copied().unwrap_or_default();
        
        println!("Frame Time - Avg: {:?}, Min: {:?}, Max: {:?}", avg, min, max);
        println!("FPS: {:.1}", 1.0 / avg.as_secs_f64());
    }
}
```

---

## 5. ANTI-PATTERN EXAMPLES

### 5.1 Avoiding Unnecessary Clones

```rust
// BAD: Cloning in hot path
fn render_items(items: Vec<Item>) -> Vec<Element> {
    items.iter().map(|item| {
        let item_clone = item.clone();  // Unnecessary clone
        render_item(item_clone)
    }).collect()
}

// GOOD: Use references
fn render_items(items: &[Item]) -> Vec<Element> {
    items.iter().map(|item| {
        render_item(item)  // No clone needed
    }).collect()
}

// GOOD: Use SmallVec for cheap clones
fn collect_ids(items: &[Item]) -> SmallVec<[Id; 8]> {
    items.iter().map(|item| item.id).collect()  // Cheap SmallVec clone
}
```

### 5.2 Avoiding Inefficient Strings

```rust
// BAD: String clones in render loop
fn render_labels(items: &[Item]) -> Vec<Element> {
    items.iter().map(|item| {
        let label = item.label.clone();  // String clone - expensive
        render_label(label)
    }).collect()
}

// GOOD: Use SharedString
fn render_labels(items: &[Item]) -> Vec<Element> {
    items.iter().map(|item| {
        let label = item.label.clone();  // SharedString clone - cheap
        render_label(label)
    }).collect()
}

// GOOD: Use references
fn render_labels(items: &[Item]) -> Vec<Element> {
    items.iter().map(|item| {
        render_label(&item.label)  // No clone needed
    }).collect()
}
```

### 5.3 Avoiding Polling

```rust
// BAD: Polling loop
fn update_loop() {
    loop {
        if state.changed() {
            render();
        }
        std::thread::sleep(Duration::from_millis(16));
    }
}

// GOOD: Event-driven
fn setup_listeners(state: &State) {
    state.on_change(|| {
        render();
    });
}

// GOOD: Frame-based updates
fn frame_loop() {
    loop {
        handle_events();
        update();
        render();
        present();
    }
}
```

