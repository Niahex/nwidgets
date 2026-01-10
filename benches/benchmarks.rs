use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

fn main() {
    divan::main();
}

// Simulate icon cache lookup
static ICON_CACHE: once_cell::sync::Lazy<RwLock<HashMap<String, Arc<str>>>> =
    once_cell::sync::Lazy::new(|| {
        let mut map = HashMap::new();
        for i in 0..100 {
            map.insert(format!("icon-{i}"), Arc::from(format!("assets/icon-{i}.svg")));
        }
        RwLock::new(map)
    });

#[divan::bench]
fn icon_cache_hit() -> Option<Arc<str>> {
    let cache = ICON_CACHE.read();
    cache.get("icon-50").cloned()
}

#[divan::bench]
fn icon_cache_miss() -> Option<Arc<str>> {
    let cache = ICON_CACHE.read();
    cache.get("nonexistent").cloned()
}

#[divan::bench]
fn json_parse_notification() -> serde_json::Value {
    let json = r#"{"app_name":"Discord","summary":"New message","body":"Hello world","urgency":1}"#;
    serde_json::from_str(json).unwrap()
}

#[divan::bench]
fn json_serialize_notification() -> String {
    let notif = serde_json::json!({
        "app_name": "Discord",
        "summary": "New message", 
        "body": "Hello world",
        "urgency": 1
    });
    serde_json::to_string(&notif).unwrap()
}

#[divan::bench]
fn string_format_dbus_path() -> String {
    let method = "ToggleChat";
    format!("org.nwidgets.App.{method}")
}

#[divan::bench]
fn hashmap_insert_and_get() -> i32 {
    let mut map = HashMap::new();
    map.insert("key1", 42);
    map.insert("key2", 100);
    *map.get("key1").unwrap()
}

#[divan::bench]
fn vec_push_and_iter(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        let mut v = Vec::with_capacity(100);
        for i in 0..100 {
            v.push(i);
        }
        v.iter().sum::<i32>()
    });
}

#[divan::bench]
fn arc_clone() -> Arc<str> {
    let s: Arc<str> = Arc::from("some cached string value");
    std::hint::black_box(s.clone())
}

// Channel operations (simulating D-Bus command dispatch)
#[divan::bench]
fn mpsc_channel_send_recv() -> i32 {
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(42).unwrap();
    rx.recv().unwrap()
}

// String operations common in UI
#[divan::bench]
fn string_truncate_display() -> String {
    let long_text = "This is a very long notification body that needs to be truncated for display";
    if long_text.len() > 50 {
        format!("{}...", &long_text[..50])
    } else {
        long_text.to_string()
    }
}

// Timestamp formatting (notifications)
#[divan::bench]
fn timestamp_format() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{}", now)
}

// Path operations (icon loading)
#[divan::bench]
fn path_join() -> std::path::PathBuf {
    let base = std::path::PathBuf::from("assets");
    base.join("icons").join("spotify.svg")
}

// Mutex lock/unlock (simulating state access)
static COUNTER: once_cell::sync::Lazy<parking_lot::Mutex<i32>> =
    once_cell::sync::Lazy::new(|| parking_lot::Mutex::new(0));

#[divan::bench]
fn mutex_lock_increment() -> i32 {
    let mut guard = COUNTER.lock();
    *guard += 1;
    *guard
}

// RwLock read vs write
static RW_DATA: once_cell::sync::Lazy<RwLock<i32>> =
    once_cell::sync::Lazy::new(|| RwLock::new(42));

#[divan::bench]
fn rwlock_read() -> i32 {
    *RW_DATA.read()
}

#[divan::bench]
fn rwlock_write() -> i32 {
    let mut guard = RW_DATA.write();
    *guard += 1;
    *guard
}

// VecDeque operations (notification history)
#[divan::bench]
fn vecdeque_push_pop(bencher: divan::Bencher) {
    bencher.bench_local(|| {
        let mut q = std::collections::VecDeque::with_capacity(50);
        for i in 0..50 {
            q.push_back(i);
        }
        while q.len() > 10 {
            q.pop_front();
        }
        q.len()
    });
}

// SharedString simulation (gpui uses this)
#[divan::bench]
fn shared_string_clone() -> Arc<str> {
    let s: Arc<str> = Arc::from("Notification title");
    let _a = s.clone();
    let _b = s.clone();
    s
}
