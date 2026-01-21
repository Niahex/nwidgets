# Optimisations Zed à Implémenter

## 1. Deferred Rendering ⚡

**Pattern Zed**: Utilise `deferred()` pour retarder le paint des éléments lourds

**À implémenter dans nwidgets**:
- Control center details (bluetooth, network, monitor)
- Notifications list
- Systray icons
- MPRIS metadata display

**Bénéfice**: Réduit le temps de frame initial, améliore la fluidité

## 2. View Caching 💾

**Pattern Zed**: `.cache()` sur les views pour éviter re-render inutiles

**À implémenter**:
- Panel modules (workspaces, datetime, network icons)
- Control center sections statiques
- OSD widget

**Bénéfice**: Évite recalcul layout/paint si rien n'a changé

## 3. Lazy Loading Amélioré 🔄

**Pattern Zed**: Cache les hauteurs d'items dans les listes

**À implémenter**:
- Control center lists (déjà lazy avec .take(), mais peut cacher hauteurs)
- Launcher results
- Notification list

## 4. Batch Updates 📦

**Pattern Zed**: Groupe les mises à jour pour réduire les re-renders

**À implémenter**:
- Audio state updates (grouper sink + source)
- Network state updates (grouper wifi + vpn + ethernet)
- System monitor (grouper CPU + RAM + GPU)

## 5. String Interning 🔤

**Pattern Zed**: Utilise `SharedString` partout (déjà fait!)

**Status**: ✅ Déjà implémenté dans panel modules

## 6. Minimal Repaints 🎨

**Pattern Zed**: Utilise `cx.notify()` seulement si vraiment changé

**À vérifier**:
- Services qui émettent des événements même si state identique
- Widgets qui re-render sans changement

## Priorités

### High Priority (Impact visible)
1. **Deferred rendering** pour control center details
2. **View caching** pour panel modules
3. **Batch updates** pour services

### Medium Priority
4. Lazy loading avec cache de hauteurs
5. Minimal repaints audit

### Low Priority
6. Micro-optimisations supplémentaires

## Implémentation

### 1. Deferred Rendering

```rust
// Dans control_center/details/monitor.rs
use gpui::deferred;

pub fn render_monitor_details(&mut self, cx: &mut Context<Self>) -> AnyElement {
    deferred(
        div()
            .bg(theme.bg)
            // ... rest of the UI
    )
    .into_any_element()
}
```

### 2. View Caching

```rust
// Dans panel/modules/datetime.rs
impl Render for DateTimeModule {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .cache() // ← Ajouter caching
            .flex()
            // ... rest
    }
}
```

### 3. Batch Updates

```rust
// Dans services/audio.rs
// Au lieu d'émettre pour chaque changement:
if sink_changed || source_changed {
    cx.emit(AudioStateChanged);
}
```

## Mesures de Performance

Avant optimisations:
- Panel render: ~1-2% CPU
- Control center open: ~2-5% CPU
- Total idle: ~0.5% CPU

Objectif après optimisations:
- Panel render: ~0.5-1% CPU (50% réduction)
- Control center open: ~1-3% CPU (40% réduction)
- Total idle: ~0.3% CPU

## Notes

- Zed utilise massivement `deferred` pour les listes longues
- Le caching de views est automatique si le state ne change pas
- Les batch updates réduisent drastiquement les re-renders
- SharedString (déjà utilisé) est crucial pour performance
