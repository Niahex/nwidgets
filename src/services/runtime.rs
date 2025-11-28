use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Shared Tokio runtime for all async operations
///
/// This provides a single multi-threaded runtime that all services can use,
/// avoiding the overhead of creating multiple runtimes.
static SHARED_RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4) // Adjust based on your needs
            .thread_name("nwidgets-tokio")
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime")
    )
});

/// Get a reference to the shared Tokio runtime
///
/// This runtime is shared across all services and is created lazily on first use.
/// It uses a multi-threaded scheduler with 4 worker threads.
///
/// # Example
/// ```
/// use crate::services::runtime;
///
/// runtime::get().spawn(async {
///     // Your async task here
/// });
/// ```
pub fn get() -> Arc<Runtime> {
    Arc::clone(&SHARED_RUNTIME)
}

/// Execute a future on the shared runtime and block until completion
///
/// This is a convenience wrapper around `runtime::get().block_on()`.
///
/// # Example
/// ```
/// use crate::services::runtime;
///
/// let result = runtime::block_on(async {
///     // Your async code here
///     42
/// });
/// ```
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    SHARED_RUNTIME.block_on(future)
}

/// Spawn a task on the shared runtime
///
/// Returns a JoinHandle that can be used to await the task's completion.
///
/// # Example
/// ```
/// use crate::services::runtime;
///
/// let handle = runtime::spawn(async {
///     // Your async task
/// });
/// ```
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    SHARED_RUNTIME.spawn(future)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_runtime() {
        let rt1 = get();
        let rt2 = get();

        // Both should point to the same runtime
        assert!(Arc::ptr_eq(&rt1, &rt2));
    }

    #[test]
    fn test_block_on() {
        let result = block_on(async { 42 });
        assert_eq!(result, 42);
    }

    #[test]
    fn test_spawn() {
        let handle = spawn(async { 100 });
        let result = block_on(handle).unwrap();
        assert_eq!(result, 100);
    }
}
