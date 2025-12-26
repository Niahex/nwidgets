use std::sync::{Arc, OnceLock};
use tokio::runtime::Runtime;

/// Runtime Tokio partagé pour toutes les opérations asynchrones.
///
/// Utilise `OnceLock` (std) au lieu de `lazy_static` pour une initialisation plus propre.
static SHARED_RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

/// Récupère une référence vers le runtime Tokio partagé.
///
/// Le runtime est initialisé lors du premier appel.
/// Il utilise un scheduler multi-threads pour gérer efficacement les tâches d'arrière-plan.
pub fn get() -> &'static Arc<Runtime> {
    SHARED_RUNTIME.get_or_init(|| {
        Arc::new(
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(4) // Suffisant pour l'UI et les services d'arrière-plan
                .thread_name("nwidgets-tokio")
                .enable_all()
                .build()
                .expect("Impossible de créer le runtime Tokio"),
        )
    })
}

/// Exécute une future sur le runtime partagé et bloque le thread courant jusqu'à la fin.
///
/// Utile pour appeler du code async depuis un contexte synchrone (ex: initialisation).
pub fn block_on<F: std::future::Future>(future: F) -> F::Output {
    get().block_on(future)
}

/// Lance une tâche bloquante sur le pool dédié de Tokio.
///
/// Idéal pour écouter des canaux `mpsc` standards ou faire des IO synchrones
/// sans bloquer les workers asynchrones et sans créer de nouveaux threads OS manuellement.
pub fn spawn_blocking<F, R>(func: F) -> tokio::task::JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    get().spawn_blocking(func)
}
