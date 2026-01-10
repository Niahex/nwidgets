use std::collections::{HashMap, VecDeque, BTreeMap, HashSet};
use std::sync::Arc;
use parking_lot::{RwLock, Mutex};

fn main() {
    divan::main();
}

// ============== CACHE OPERATIONS ==============

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
    ICON_CACHE.read().get("icon-50").cloned()
}

#[divan::bench]
fn icon_cache_miss() -> Option<Arc<str>> {
    ICON_CACHE.read().get("nonexistent").cloned()
}

// ============== JSON OPERATIONS ==============

#[divan::bench]
fn json_parse_small() -> serde_json::Value {
    serde_json::from_str(r#"{"a":1}"#).unwrap()
}

#[divan::bench]
fn json_parse_notification() -> serde_json::Value {
    serde_json::from_str(r#"{"app_name":"Discord","summary":"New message","body":"Hello world","urgency":1}"#).unwrap()
}

#[divan::bench]
fn json_parse_large() -> serde_json::Value {
    serde_json::from_str(r#"{"notifications":[{"app":"Discord","title":"Msg 1"},{"app":"Slack","title":"Msg 2"},{"app":"Teams","title":"Msg 3"}],"count":3}"#).unwrap()
}

#[divan::bench]
fn json_serialize_small() -> String {
    serde_json::to_string(&serde_json::json!({"a":1})).unwrap()
}

#[divan::bench]
fn json_serialize_notification() -> String {
    serde_json::to_string(&serde_json::json!({"app_name":"Discord","summary":"New message","body":"Hello","urgency":1})).unwrap()
}

// ============== STRING OPERATIONS ==============

#[divan::bench]
fn string_format_simple() -> String {
    format!("Hello {}", "world")
}

#[divan::bench]
fn string_format_multiple() -> String {
    format!("{} {} {} {}", "one", "two", "three", "four")
}

#[divan::bench]
fn string_concat_push() -> String {
    let mut s = String::with_capacity(50);
    s.push_str("Hello ");
    s.push_str("world");
    s
}

#[divan::bench]
fn string_truncate_short() -> String {
    let text = "Short";
    if text.len() > 50 { format!("{}...", &text[..50]) } else { text.to_string() }
}

#[divan::bench]
fn string_truncate_long() -> String {
    let text = "This is a very long notification body that definitely needs truncation";
    if text.len() > 50 { format!("{}...", &text[..50]) } else { text.to_string() }
}

#[divan::bench]
fn string_replace() -> String {
    "Hello <name>, welcome to <app>".replace("<name>", "User").replace("<app>", "NWidgets")
}

#[divan::bench]
fn string_split_collect() -> Vec<&'static str> {
    "one,two,three,four,five".split(',').collect()
}

#[divan::bench]
fn string_contains() -> bool {
    "This is a long string with some content".contains("content")
}

#[divan::bench]
fn string_starts_with() -> bool {
    "/org/freedesktop/Notifications".starts_with("/org/freedesktop")
}

// ============== COLLECTIONS ==============

#[divan::bench]
fn hashmap_insert_10() -> HashMap<i32, i32> {
    let mut m = HashMap::new();
    for i in 0..10 { m.insert(i, i * 2); }
    m
}

#[divan::bench]
fn hashmap_lookup() -> Option<i32> {
    static MAP: once_cell::sync::Lazy<HashMap<i32, i32>> = once_cell::sync::Lazy::new(|| {
        (0..100).map(|i| (i, i * 2)).collect()
    });
    MAP.get(&50).copied()
}

#[divan::bench]
fn btreemap_insert_10() -> BTreeMap<i32, i32> {
    let mut m = BTreeMap::new();
    for i in 0..10 { m.insert(i, i * 2); }
    m
}

#[divan::bench]
fn hashset_insert_contains() -> bool {
    let mut s = HashSet::new();
    for i in 0..10 { s.insert(i); }
    s.contains(&5)
}

#[divan::bench]
fn vec_push_100() -> Vec<i32> {
    let mut v = Vec::with_capacity(100);
    for i in 0..100 { v.push(i); }
    v
}

#[divan::bench]
fn vec_iter_sum() -> i32 {
    (0..100).collect::<Vec<_>>().iter().sum()
}

#[divan::bench]
fn vec_filter_map() -> Vec<i32> {
    (0..100).filter(|x| x % 2 == 0).map(|x| x * 2).collect()
}

#[divan::bench]
fn vecdeque_push_pop_50() -> usize {
    let mut q = VecDeque::with_capacity(50);
    for i in 0..50 { q.push_back(i); }
    while q.len() > 10 { q.pop_front(); }
    q.len()
}

#[divan::bench]
fn vecdeque_rotate() -> i32 {
    let mut q: VecDeque<i32> = (0..20).collect();
    q.rotate_left(5);
    q[0]
}

// ============== SYNC PRIMITIVES ==============

static MUTEX_COUNTER: once_cell::sync::Lazy<Mutex<i32>> = once_cell::sync::Lazy::new(|| Mutex::new(0));
static RWLOCK_DATA: once_cell::sync::Lazy<RwLock<i32>> = once_cell::sync::Lazy::new(|| RwLock::new(42));

#[divan::bench]
fn mutex_lock() -> i32 {
    *MUTEX_COUNTER.lock()
}

#[divan::bench]
fn mutex_lock_modify() -> i32 {
    let mut g = MUTEX_COUNTER.lock();
    *g += 1;
    *g
}

#[divan::bench]
fn rwlock_read() -> i32 {
    *RWLOCK_DATA.read()
}

#[divan::bench]
fn rwlock_write() -> i32 {
    *RWLOCK_DATA.write() += 1;
    *RWLOCK_DATA.read()
}

#[divan::bench]
fn arc_clone() -> Arc<str> {
    let s: Arc<str> = Arc::from("cached value");
    std::hint::black_box(s.clone())
}

#[divan::bench]
fn arc_clone_3x() -> Arc<str> {
    let s: Arc<str> = Arc::from("cached value");
    let _ = s.clone();
    let _ = s.clone();
    s.clone()
}

// ============== CHANNELS ==============

#[divan::bench]
fn mpsc_send_recv() -> i32 {
    let (tx, rx) = std::sync::mpsc::channel();
    tx.send(42).unwrap();
    rx.recv().unwrap()
}

#[divan::bench]
fn mpsc_try_recv_empty() -> Option<i32> {
    let (_tx, rx) = std::sync::mpsc::channel::<i32>();
    rx.try_recv().ok()
}

// ============== PATH OPERATIONS ==============

#[divan::bench]
fn path_join_2() -> std::path::PathBuf {
    std::path::PathBuf::from("assets").join("icon.svg")
}

#[divan::bench]
fn path_join_3() -> std::path::PathBuf {
    std::path::PathBuf::from("home").join("user").join("assets")
}

#[divan::bench]
fn path_extension() -> Option<&'static std::ffi::OsStr> {
    std::path::Path::new("icon.svg").extension()
}

#[divan::bench]
fn path_file_name() -> Option<&'static std::ffi::OsStr> {
    std::path::Path::new("/home/user/icon.svg").file_name()
}

// ============== TIME OPERATIONS ==============

#[divan::bench]
fn time_now() -> std::time::Instant {
    std::time::Instant::now()
}

#[divan::bench]
fn time_duration_since() -> std::time::Duration {
    static START: once_cell::sync::Lazy<std::time::Instant> = once_cell::sync::Lazy::new(std::time::Instant::now);
    START.elapsed()
}

#[divan::bench]
fn timestamp_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs()
}

#[divan::bench]
fn timestamp_format() -> String {
    format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs())
}

// ============== OPTION/RESULT ==============

#[divan::bench]
fn option_unwrap_or() -> i32 {
    let x: Option<i32> = None;
    x.unwrap_or(42)
}

#[divan::bench]
fn option_map() -> Option<i32> {
    Some(21).map(|x| x * 2)
}

#[divan::bench]
fn result_map_err() -> Result<i32, &'static str> {
    let r: Result<i32, i32> = Err(404);
    r.map_err(|_| "not found")
}

// ============== NUMERIC ==============

#[divan::bench]
fn float_lerp() -> f32 {
    let a = 0.0f32;
    let b = 100.0f32;
    let t = 0.5f32;
    a + (b - a) * t
}

#[divan::bench]
fn float_clamp() -> f32 {
    let v = 150.0f32;
    v.clamp(0.0, 100.0)
}

#[divan::bench]
fn int_parse() -> i32 {
    "12345".parse().unwrap()
}

#[divan::bench]
fn int_to_string() -> String {
    12345.to_string()
}

// ============== ALLOCATION ==============

#[divan::bench]
fn box_alloc() -> Box<[u8; 1024]> {
    Box::new([0u8; 1024])
}

#[divan::bench]
fn vec_alloc_1kb() -> Vec<u8> {
    vec![0u8; 1024]
}

#[divan::bench]
fn string_alloc_100() -> String {
    String::with_capacity(100)
}
