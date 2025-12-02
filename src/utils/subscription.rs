use async_channel;
use glib::MainContext;
use std::sync::mpsc;

/// Helper générique pour connecter des threads de monitoring synchrones à l'interface GTK.
pub struct ServiceSubscription;

impl ServiceSubscription {
    /// Abonne un callback aux mises à jour d'un Receiver.
    ///
    /// Cette version utilise `async_channel` et `spawn_local` pour permettre
    /// l'utilisation de callbacks qui ne sont pas thread-safe (ex: mise à jour de widgets GTK).
    ///
    /// Le `callback` n'a PAS besoin d'être `Send` ni `Sync`.
    pub fn subscribe<T, F>(rx: mpsc::Receiver<T>, callback: F)
    where
        T: Send + 'static,
        F: Fn(T) + 'static,
    {
        // Canal intermédiaire pour passer du monde sync (mpsc/thread de fond)
        // au monde async (MainContext/thread UI)
        let (async_tx, async_rx) = async_channel::unbounded();

        // Tâche de fond : Pont mpsc -> async_channel
        // Utilise le pool de threads partagé via spawn_blocking au lieu de créer un thread OS dédié.
        // Cela réduit l'empreinte mémoire si on a beaucoup de widgets.
        crate::utils::runtime::spawn_blocking(move || {
            while let Ok(msg) = rx.recv() {
                if async_tx.send_blocking(msg).is_err() {
                    break;
                }
            }
        });

        // Tâche principale : Lecture async_channel -> callback
        // `spawn_local` garantit l'exécution sur le thread principal GTK (le thread qui a initialisé le contexte),
        // ce qui est requis pour manipuler l'UI.
        MainContext::default().spawn_local(async move {
            while let Ok(msg) = async_rx.recv().await {
                callback(msg);
            }
        });
    }
}
