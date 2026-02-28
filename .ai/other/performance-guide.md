# nwidgets - Guide des Optimisations de Performance

## 🎯 Résumé Exécutif

**Réduction de 90% du CPU en idle: 5% → 0.5%**

Ce document sert de guide de référence pour maintenir et améliorer les performances de nwidgets.

## 📋 Checklist des Optimisations

### ✅ Implémenté

#### Architecture
- [x] Event-driven avec `tokio::Notify` au lieu de polling
- [x] State comparison avant `cx.emit()` dans tous les services
- [x] On-demand monitoring (pause quand non utilisé)
- [x] Événements séparés au lieu de debouncing

#### UI Rendering
- [x] Deferred rendering pour vues complexes
- [x] Lazy loading avec `.take(N)` sur toutes les listes
- [x] SharedString pour cache de strings UI
- [x] Élimination des clones inutiles

#### Structure
- [x] Modularisation du control center (12 fichiers)
- [x] Séparation des responsabilités
- [x] Code maintenable (<200 lignes par fichier)

## 🔧 Patterns à Suivre

### 1. Event-Driven Architecture

**Quand l'utiliser**: Pour tout ce qui peut être notifié par événement

**Pattern**:
```rust
use tokio::sync::Notify;
use std::sync::Arc;

struct Service {
    notify: Arc<Notify>,
}

// Worker thread
async fn worker(notify: Arc<Notify>) {
    loop {
        notify.notified().await; // Bloque avec 0% CPU
        // Process event
    }
}

// Trigger
notify.notify_one();
```

**Exemples dans nwidgets**:
- MPRIS: Détection Spotify via Hyprland events
- System Monitor: Pause/resume via Notify
- Active Window: Update cache seulement sur événement

### 2. State Comparison

**Quand l'utiliser**: Dans tous les services qui émettent des événements

**Pattern**:
```rust
let changed = {
    let mut current = self.state.write();
    if *current != new_state {
        *current = new_state;
        true
    } else {
        false
    }
};

if changed {
    cx.emit(StateChanged);
    cx.notify();
}
```

**Bénéfice**: Évite re-renders inutiles si state identique

**Exemples dans nwidgets**:
- AudioService
- NetworkService
- SystemMonitor

### 3. Deferred Rendering

**Quand l'utiliser**: Pour vues complexes qui ne sont pas toujours visibles

**Pattern**:
```rust
use gpui::deferred;

fn render_complex_view(&mut self, cx: &mut Context<Self>) -> AnyElement {
    deferred(
        div()
            .bg(theme.bg)
            .child(/* complex UI */)
    )
    .into_any_element()
}
```

**Bénéfice**: Retarde le paint, améliore frame time initial

**Exemples dans nwidgets**:
- Control center details (bluetooth, network, monitor, sink, source)

### 4. Lazy Loading

**Quand l'utiliser**: Pour toutes les listes qui peuvent être longues

**Pattern**:
```rust
.children(items.iter().take(MAX_ITEMS).map(|item| {
    // render item
}))
```

**Limites recommandées**:
- Bluetooth devices: 8
- VPN connections: 6
- Disk mounts: 7
- Audio streams: 5
- Notifications: 5
- Launcher results: 10

**Bénéfice**: Évite rendu de centaines d'items inutiles

### 5. SharedString Caching

**Quand l'utiliser**: Pour strings UI qui changent rarement

**Pattern**:
```rust
use gpui::SharedString;

struct Module {
    cached_text: SharedString,
}

// Update seulement sur événement
fn on_event(&mut self, new_text: String) {
    self.cached_text = new_text.into();
}

// Render avec clone gratuit
fn render(&self) -> impl IntoElement {
    div().child(self.cached_text.clone())
}
```

**Bénéfice**: Clone gratuit (Arc-based), pas de réallocation

**Exemples dans nwidgets**:
- Active Window: icon, class, title
- MPRIS: title, artist, status

### 6. On-Demand Monitoring

**Quand l'utiliser**: Pour monitoring qui n'est pas toujours nécessaire

**Pattern**:
```rust
use tokio::sync::Notify;

async fn monitor_loop(notify: Arc<Notify>) {
    loop {
        tokio::select! {
            _ = tokio::time::sleep(interval) => {
                // Collect stats
            }
            _ = notify.notified() => {
                // Pause monitoring
                notify.notified().await; // Wait for resume
            }
        }
    }
}
```

**Bénéfice**: 0% CPU quand non utilisé

**Exemples dans nwidgets**:
- System Monitor: Pause quand control center fermé

## 🚫 Anti-Patterns à Éviter

### ❌ Polling Inutile

**Mauvais**:
```rust
loop {
    tokio::time::sleep(Duration::from_millis(100)).await;
    check_state();
}
```

**Bon**:
```rust
loop {
    notify.notified().await; // Event-driven
    check_state();
}
```

### ❌ Émission Sans Vérification

**Mauvais**:
```rust
self.state = new_state;
cx.emit(StateChanged);
cx.notify();
```

**Bon**:
```rust
if self.state != new_state {
    self.state = new_state;
    cx.emit(StateChanged);
    cx.notify();
}
```

### ❌ Clones Inutiles

**Mauvais**:
```rust
let theme = cx.theme().clone();
// ... pas d'appels mutables après
render_with_theme(&theme);
```

**Bon**:
```rust
let theme = cx.theme();
render_with_theme(&theme);
```

**Exception**: Clone nécessaire si appels mutables après:
```rust
let theme = cx.theme().clone();
self.audio.update(cx, |audio, cx| { /* mutable */ });
render_with_theme(&theme); // theme utilisé après appel mutable
```

### ❌ Listes Sans Limite

**Mauvais**:
```rust
.children(devices.iter().map(|device| {
    // render device
}))
```

**Bon**:
```rust
.children(devices.iter().take(8).map(|device| {
    // render device
}))
```

## 📊 Monitoring des Performances

### Métriques à Surveiller

1. **CPU Usage (Idle)**
   - Target: <1%
   - Mesure: `top` ou `htop`
   - Fréquence: Après chaque changement majeur

2. **Memory Usage**
   - Target: <100MB
   - Mesure: `top` ou `htop`
   - Fréquence: Tests de longue durée

3. **Frame Time**
   - Target: <16ms (60 FPS)
   - Mesure: GPUI profiler
   - Fréquence: Tests UI intensifs

### Outils de Profiling

1. **perf** (Linux)
   ```bash
   perf record -g ./target/release/nwidgets
   perf report
   ```

2. **flamegraph**
   ```bash
   cargo install flamegraph
   cargo flamegraph --bin nwidgets
   ```

3. **GPUI Profiler**
   ```rust
   // Dans le code
   cx.profile("operation_name", || {
       // code to profile
   });
   ```

## 🔍 Debugging Performance Issues

### Symptômes et Solutions

#### CPU élevé en idle
1. Vérifier les loops avec `tokio::time::sleep`
2. Chercher les `cx.notify()` appelés trop souvent
3. Profiler avec `perf` pour trouver hot spots

#### UI qui lag
1. Vérifier les listes sans `.take(N)`
2. Ajouter `deferred()` sur vues complexes
3. Vérifier les calculs lourds dans `render()`

#### Memory leak
1. Vérifier les `Arc` qui ne sont pas dropped
2. Chercher les vecs qui grandissent indéfiniment
3. Utiliser `valgrind` pour détecter leaks

## 📚 Références

### Documentation Interne
- `performance-estimation.md`: Analyse détaillée des optimisations
- `zed-optimizations.md`: Patterns Zed implémentés
- `optimization-summary.md`: Résumé complet

### Code Examples
- `src/services/mpris.rs`: Event-driven avec Notify
- `src/services/audio.rs`: State comparison
- `src/widgets/control_center/details/`: Deferred rendering
- `src/widgets/panel/modules/active_window.rs`: SharedString cache

### External Resources
- [GPUI Performance Guide](https://github.com/zed-industries/zed)
- [Tokio Best Practices](https://tokio.rs/tokio/tutorial)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

## 🎓 Leçons Apprises

### 1. Event-Driven > Polling
Le plus grand gain de performance vient du remplacement du polling par des événements. Toujours privilégier les événements quand possible.

### 2. Mesurer Avant d'Optimiser
Utiliser `perf` et profiling pour identifier les vrais bottlenecks. Ne pas optimiser prématurément.

### 3. State Comparison est Crucial
Vérifier si le state a changé avant d'émettre évite des cascades de re-renders inutiles.

### 4. Lazy Loading est Essentiel
Limiter le nombre d'items rendus est plus efficace que d'optimiser le rendu de chaque item.

### 5. SharedString pour UI
Pour les strings affichées dans l'UI, SharedString offre des clones gratuits et réduit les allocations.

## 🚀 Optimisations Futures

### Possibles mais Non Prioritaires

1. **GPU Acceleration**
   - Utiliser GPUI paint layers
   - Offload rendering au GPU
   - Impact: Moyen, Complexité: Élevée

2. **Incremental Rendering**
   - Update seulement parties changées
   - Diff-based rendering
   - Impact: Faible, Complexité: Élevée

3. **Background Loading**
   - Précharger données en background
   - Lazy initialization
   - Impact: Faible, Complexité: Moyenne

4. **Memory Pooling**
   - Réutiliser allocations
   - Object pools
   - Impact: Faible, Complexité: Moyenne

### Quand Optimiser Davantage

- Si CPU idle > 1%
- Si frame time > 16ms régulièrement
- Si memory usage > 150MB
- Si feedback utilisateur sur lag

## ✅ Checklist pour Nouveaux Features

Avant d'ajouter un nouveau feature, vérifier:

- [ ] Utilise event-driven au lieu de polling si possible
- [ ] Compare state avant d'émettre événements
- [ ] Limite les listes avec `.take(N)`
- [ ] Utilise SharedString pour strings UI
- [ ] Ajoute `deferred()` si vue complexe
- [ ] Évite clones inutiles
- [ ] Profile le CPU usage
- [ ] Teste le frame time

## 📝 Conclusion

Ce guide doit être consulté lors de:
- Ajout de nouveaux features
- Debugging de problèmes de performance
- Code reviews
- Refactoring majeur

**Objectif**: Maintenir <1% CPU idle et 60 FPS constant.
