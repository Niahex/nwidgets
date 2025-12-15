use once_cell::sync::Lazy;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Shared Tokio runtime for all background services
/// This avoids creating multiple runtimes and provides a single thread pool
static SHARED_RUNTIME: Lazy<Arc<Runtime>> = Lazy::new(|| {
    Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(4)
            .thread_name("nwidgets-tokio")
            .enable_all()
            .build()
            .expect("Failed to create Tokio runtime"),
    )
});

/// Get a reference to the shared Tokio runtime
pub fn get() -> &'static Arc<Runtime> {
    &SHARED_RUNTIME
}

/// Block on a future using the shared runtime
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    get().block_on(future)
}

/// Spawn a future on the shared runtime
pub fn spawn<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: std::future::Future + Send + 'static,
    F::Output: Send + 'static,
{
    get().spawn(future)
}

/// Spawn a blocking task on the shared runtime
pub fn spawn_blocking<F, R>(func: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    get().spawn_blocking(func)
}
