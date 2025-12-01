use async_channel;
use glib::MainContext;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

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

    /// Crée un système de souscription avec un moniteur centralisé.
    ///
    /// Utile pour les services qui ont plusieurs abonnés (ex: HyprlandService).
    /// Le `monitor_fn` est exécuté une seule fois.
    pub fn create_subscription_system<T, M>(monitor_fn: M) -> impl Fn(Box<dyn Fn(T) + 'static>)
    where
        T: Clone + Send + 'static,
        M: FnOnce(mpsc::Sender<T>) + Send + 'static,
    {
        let subscribers: Arc<Mutex<Vec<mpsc::Sender<T>>>> = Arc::new(Mutex::new(Vec::new()));
        let subscribers_clone = Arc::clone(&subscribers);

        // Lancer le monitoring dans le pool de threads partagé
        crate::utils::runtime::spawn_blocking(move || {
            let (tx, rx) = mpsc::channel();

            // Exécuter la fonction de monitoring (qui peut être bloquante ou lancer son propre processus)
            monitor_fn(tx);

            // Dispatcher les événements à tous les abonnés
            while let Ok(state) = rx.recv() {
                let mut subs = subscribers_clone.lock().unwrap();
                subs.retain(|subscriber| subscriber.send(state.clone()).is_ok());
            }
        });

        // Retourne la closure d'abonnement que les widgets utiliseront
        move |callback: Box<dyn Fn(T) + 'static>| {
            let (tx, rx) = mpsc::channel();
            subscribers.lock().unwrap().push(tx);
            // On délègue à subscribe qui gère le pont vers le thread principal
            Self::subscribe(rx, callback);
        }
    }

    /// Helper pour les services basés sur du polling (vérification périodique).
    ///
    /// Lance une boucle infinie qui vérifie l'état toutes les `interval` et notifie en cas de changement.
    pub fn create_polling_subscription<T, F>(
        poll_fn: F,
        interval: Duration,
    ) -> impl Fn(Box<dyn Fn(T) + 'static>)
    where
        T: Clone + Send + PartialEq + 'static,
        F: Fn() -> T + Send + Sync + 'static,
    {
        Self::create_subscription_system(move |tx| {
            // Le polling tourne dans le spawn_blocking de create_subscription_system
            let mut last_state = poll_fn();
            let _ = tx.send(last_state.clone());

            loop {
                thread::sleep(interval);
                let current_state = poll_fn();

                if current_state != last_state {
                    last_state = current_state.clone();
                    if tx.send(current_state).is_err() {
                        break; // Arrêt si plus personne n'écoute (canal fermé)
                    }
                }
            }
        })
    }
}
