# Patterns de Robustesse de Zed

## 🛡️ Analyse de la Gestion d'Erreurs

### Patterns Identifiés dans Zed

#### 1. `.log_err()` - Pattern Principal ⭐⭐⭐⭐⭐

**Usage Zed**: 1167+ occurrences

```rust
// Au lieu de .unwrap() ou panic
result.log_err();

// Avec Option
some_option.log_err()?;

// Dans les tasks
task.await.log_err();
```

**Bénéfice**: 
- Log l'erreur automatiquement
- Continue l'exécution
- Pas de panic

#### 2. `Result<T>` et `anyhow::Result` ⭐⭐⭐⭐⭐

**Usage Zed**: 9973+ occurrences

```rust
use anyhow::{Result, Context};

fn operation() -> Result<T> {
    something()
        .context("Failed to do something")?
}
```

#### 3. `if let Err(e)` avec Logging ⭐⭐⭐⭐

```rust
if let Err(err) = result {
    log::error!("Operation failed: {err}");
}
```

#### 4. `.unwrap_or_else()` avec Fallback ⭐⭐⭐⭐

```rust
value.unwrap_or_else(|| {
    log::warn!("Using default value");
    default_value
})
```

#### 5. `log::error!` et `log::warn!` ⭐⭐⭐⭐⭐

**Usage Zed**: 1055+ occurrences

```rust
log::error!("Failed to connect: {}", e);
log::warn!("Retrying operation");
```

## 📊 État Actuel de nwidgets

### Problèmes Identifiés

#### 1. Trop de `eprintln!` et `println!` (72 occurrences)

**Fichiers concernés**:
- `launcher.rs`: 16 eprintln
- `mpris.rs`: 11 eprintln
- `dbus.rs`: 8 eprintln
- `bluetooth.rs`: 8 eprintln
- `main.rs`: 8 eprintln
- `notifications.rs`: 6 (println + eprintln)
- `audio.rs`: 5 eprintln
- `system_monitor.rs`: 3 eprintln
- `network/network.rs`: 2 eprintln
- `hyprland.rs`: 2 eprintln
- `clipboard.rs`: 1 eprintln
- `control_center/mod.rs`: 1 eprintln
- `utils/icon.rs`: 1 eprintln

**Problème**: 
- Pas de niveaux de log (error, warn, info, debug)
- Difficile à filtrer
- Pas de timestamps
- Pas de contexte

#### 2. Quelques `.unwrap()` Restants (12 occurrences)

**Fichiers**:
- `launcher/applications.rs`: 3 unwrap (Mutex)
- `main.rs`: 3 unwrap
- `dbus.rs`: 2 unwrap
- `cef/browser.rs`: 2 (1 expect, 1 unwrap)
- `notifications.rs`: 2 unwrap

#### 3. Pas de Logging Structuré

Pas de crate `log` ou `tracing`

## 🎯 Recommandations

### 1. Ajouter `log` Crate ⭐⭐⭐⭐⭐

**Priorité**: Haute

```toml
# Cargo.toml
[dependencies]
log = "0.4"
env_logger = "0.11"
```

```rust
// main.rs
fn main() {
    env_logger::init();
    // ...
}
```

### 2. Remplacer `eprintln!` par `log::*` ⭐⭐⭐⭐⭐

**Avant**:
```rust
eprintln!("[MPRIS] Failed to connect: {}", e);
```

**Après**:
```rust
log::error!("Failed to connect to MPRIS: {}", e);
```

**Niveaux**:
- `log::error!`: Erreurs critiques
- `log::warn!`: Avertissements
- `log::info!`: Informations importantes
- `log::debug!`: Debug (désactivé en release)
- `log::trace!`: Trace détaillée

### 3. Créer Extension Trait `.log_err()` ⭐⭐⭐⭐

```rust
// src/utils/result_ext.rs
pub trait ResultExt<T> {
    fn log_err(self) -> Option<T>;
}

impl<T, E: std::fmt::Display> ResultExt<T> for Result<T, E> {
    fn log_err(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                log::error!("{}", e);
                None
            }
        }
    }
}
```

**Usage**:
```rust
// Au lieu de
match result {
    Ok(v) => v,
    Err(e) => {
        eprintln!("Error: {}", e);
        return;
    }
}

// Utiliser
let Some(value) = result.log_err() else {
    return;
};
```

### 4. Remplacer `.unwrap()` Dangereux ⭐⭐⭐

**Mutex unwrap** (OK - panic si poisoned):
```rust
// Garder tel quel - panic voulu si mutex poisoned
let mut apps = applications.lock().unwrap();
```

**Autres unwrap** (À remplacer):
```rust
// main.rs - Avant
.unwrap();

// Après
.expect("Failed to initialize application");
// ou
.log_err();
```

### 5. Ajouter Contexte aux Erreurs ⭐⭐⭐

```rust
use anyhow::{Context, Result};

fn load_config() -> Result<Config> {
    std::fs::read_to_string("config.toml")
        .context("Failed to read config file")?;
    // ...
}
```

## 📋 Plan d'Implémentation

### Phase 1: Infrastructure (Priorité Haute)

1. **Ajouter dépendances**
   ```toml
   log = "0.4"
   env_logger = "0.11"
   anyhow = "1.0"
   ```

2. **Créer `utils/result_ext.rs`**
   - Trait `ResultExt` avec `.log_err()`
   - Trait `OptionExt` avec `.log_none()`

3. **Initialiser logger dans `main.rs`**
   ```rust
   env_logger::Builder::from_default_env()
       .filter_level(log::LevelFilter::Info)
       .init();
   ```

### Phase 2: Migration (Priorité Moyenne)

4. **Remplacer `eprintln!` par `log::*`** (72 occurrences)
   - Services: error/warn
   - Debug: debug/trace
   - Info: info

5. **Remplacer `.unwrap()` non-Mutex** (9 occurrences)
   - `main.rs`: `.expect()` avec message
   - `dbus.rs`: `.log_err()`
   - `cef/browser.rs`: `.log_err()`
   - `notifications.rs`: `.log_err()`

### Phase 3: Amélioration (Priorité Basse)

6. **Ajouter `anyhow::Result` aux fonctions**
   - Fonctions qui peuvent fail
   - Propagation d'erreurs avec `?`

7. **Ajouter contexte aux erreurs**
   - `.context()` sur les opérations I/O
   - Messages d'erreur descriptifs

## 🔍 Exemples de Migration

### Exemple 1: MPRIS Service

**Avant**:
```rust
eprintln!("[MPRIS] Failed to connect to session bus: {}", e);
```

**Après**:
```rust
log::error!("Failed to connect to MPRIS session bus: {}", e);
```

### Exemple 2: Launcher

**Avant**:
```rust
eprintln!("[nlauncher] Launching: {name} with command: {exec}");
match result {
    Ok(_) => eprintln!("[nlauncher] Successfully launched: {name}"),
    Err(err) => eprintln!("[nlauncher] Failed to launch {name}: {err}"),
}
```

**Après**:
```rust
log::info!("Launching application: {name} with command: {exec}");
match result {
    Ok(_) => log::info!("Successfully launched: {name}"),
    Err(err) => log::error!("Failed to launch {name}: {err}"),
}
```

### Exemple 3: Unwrap Dangereux

**Avant**:
```rust
let event = self.current_event.as_ref().unwrap();
```

**Après** (déjà fait):
```rust
let Some(event) = self.current_event.as_ref() else {
    return div();
};
```

### Exemple 4: Avec `.log_err()`

**Avant**:
```rust
match connection.send(message) {
    Ok(_) => {},
    Err(e) => eprintln!("Failed to send: {}", e),
}
```

**Après**:
```rust
connection.send(message).log_err();
```

## 📊 Impact Estimé

### Bénéfices

1. **Robustesse**: ⭐⭐⭐⭐⭐
   - Moins de panics potentiels
   - Meilleure gestion d'erreurs
   - Récupération gracieuse

2. **Debuggabilité**: ⭐⭐⭐⭐⭐
   - Logs structurés avec niveaux
   - Filtrage par niveau (RUST_LOG=debug)
   - Timestamps automatiques
   - Contexte des erreurs

3. **Maintenabilité**: ⭐⭐⭐⭐
   - Code plus propre
   - Pattern uniforme
   - Facile à comprendre

### Effort

- **Phase 1** (Infrastructure): 1-2h
- **Phase 2** (Migration): 2-3h
- **Phase 3** (Amélioration): 1-2h

**Total**: 4-7h de travail

### Priorités

1. **Haute**: Phase 1 + Migration eprintln → log
2. **Moyenne**: Migration unwrap → safe alternatives
3. **Basse**: anyhow::Result et contexte

## ✅ Checklist

### Infrastructure
- [ ] Ajouter `log`, `env_logger`, `anyhow` à Cargo.toml
- [ ] Créer `utils/result_ext.rs` avec `.log_err()`
- [ ] Initialiser logger dans `main.rs`

### Migration Services (72 eprintln)
- [ ] `launcher.rs` (16)
- [ ] `mpris.rs` (11)
- [ ] `dbus.rs` (8)
- [ ] `bluetooth.rs` (8)
- [ ] `main.rs` (8)
- [ ] `notifications.rs` (6)
- [ ] `audio.rs` (5)
- [ ] `system_monitor.rs` (3)
- [ ] `network/network.rs` (2)
- [ ] `hyprland.rs` (2)
- [ ] `clipboard.rs` (1)
- [ ] `control_center/mod.rs` (1)
- [ ] `utils/icon.rs` (1)

### Migration Unwrap (9 non-Mutex)
- [ ] `main.rs` (3)
- [ ] `dbus.rs` (2)
- [ ] `cef/browser.rs` (2)
- [ ] `notifications.rs` (2)

## 🎓 Leçons de Zed

1. **`.log_err()` est le pattern par défaut** pour gérer les erreurs non-critiques
2. **`log::error!` et `log::warn!`** pour logging structuré
3. **`anyhow::Result`** pour propagation d'erreurs avec contexte
4. **Éviter `unwrap()`** sauf pour Mutex (panic voulu si poisoned)
5. **Logging structuré** > `println!` / `eprintln!`

## 📝 Conclusion

**Robustesse actuelle**: ⭐⭐⭐ (Bonne mais améliorable)

**Après migration**: ⭐⭐⭐⭐⭐ (Excellente)

**Priorité**: Haute (améliore debuggabilité et robustesse)

**Effort**: Moyen (4-7h)

**Impact**: Élevé (meilleure expérience développeur et utilisateur)
