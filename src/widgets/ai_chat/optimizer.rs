use super::types::{ChatMessage, MessageRole};

/// Compression intelligente qui préserve la substance du message
pub fn compress_message(msg: &ChatMessage) -> ChatMessage {
    let compressed_content = if msg.content.len() > 300 {
        compress_intelligently(&msg.content)
    } else {
        msg.content.clone()
    };

    ChatMessage {
        role: msg.role.clone(),
        content: compressed_content,
    }
}

/// Compression intelligente basée sur l'analyse du contenu
fn compress_intelligently(content: &str) -> String {
    // Détecter si c'est du code
    if is_code_content(content) {
        return compress_code_content(content);
    }

    // Extraire les phrases et les scorer
    let sentences: Vec<&str> = content
        .split(&['.', '!', '?'][..])
        .filter(|s| !s.trim().is_empty())
        .collect();

    if sentences.is_empty() {
        return content.chars().take(300).collect();
    }

    // Calculer longueur cible selon longueur originale
    let target_length = match content.len() {
        0..=300 => content.len(),
        301..=600 => 300,
        601..=1000 => 400,
        1001..=2000 => 500,
        _ => 600,
    };

    // Scorer chaque phrase
    let mut scored: Vec<(f32, usize, &str)> = sentences
        .iter()
        .enumerate()
        .map(|(idx, sentence)| {
            let score = score_sentence_importance(sentence, idx, sentences.len());
            (score, idx, *sentence)
        })
        .collect();

    // Trier par score décroissant mais garder l'ordre original ensuite
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    // Sélectionner phrases importantes jusqu'à longueur cible
    let mut selected_indices = Vec::new();
    let mut current_len = 0;

    for (_, idx, sentence) in scored.iter() {
        let sentence_len = sentence.len();
        if current_len + sentence_len <= target_length || selected_indices.is_empty() {
            selected_indices.push(*idx);
            current_len += sentence_len;
        }
    }

    // Remettre dans l'ordre original
    selected_indices.sort();

    // Reconstruire le texte
    let mut result = String::new();
    for (i, &idx) in selected_indices.iter().enumerate() {
        if let Some(sentence) = sentences.get(idx) {
            result.push_str(sentence.trim());
            // Ajouter ponctuation si manquante
            if !result.ends_with(&['.', '!', '?'][..]) {
                result.push('.');
            }
            if i < selected_indices.len() - 1 {
                result.push(' ');
            }
        }
    }

    // Ajouter indicateur si compression significative
    if result.len() < content.len() / 2 {
        result.push_str(" [...]");
    }

    result
}

/// Détecte si le contenu est du code
fn is_code_content(content: &str) -> bool {
    content.contains("```") ||
    content.contains("fn ") ||
    content.contains("pub struct") ||
    content.contains("impl ") ||
    content.lines().filter(|l| l.trim().starts_with("//")).count() > 2
}

/// Compresse du code en gardant les signatures importantes
fn compress_code_content(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut important = Vec::new();

    for line in lines.iter() {
        let trimmed = line.trim();
        // Garder les déclarations importantes
        if trimmed.starts_with("fn ") ||
           trimmed.starts_with("pub fn") ||
           trimmed.starts_with("struct ") ||
           trimmed.starts_with("pub struct") ||
           trimmed.starts_with("impl ") ||
           trimmed.starts_with("use ") ||
           trimmed.starts_with("//") ||
           trimmed.contains("ERROR") ||
           trimmed.contains("TODO") {
            important.push(*line);
        }
    }

    if important.is_empty() {
        // Fallback: prendre début et fin
        let take = lines.len().min(10);
        let mut result: Vec<&str> = lines.iter().take(take).copied().collect();
        if lines.len() > take * 2 {
            result.push("    // ...");
            result.extend(lines.iter().rev().take(5).rev());
        }
        result.join("\n")
    } else {
        important.join("\n")
    }
}

/// Score l'importance d'une phrase
fn score_sentence_importance(sentence: &str, position: usize, total: usize) -> f32 {
    let mut score = 0.0;

    // 1. Position (début et fin plus importants)
    if position == 0 {
        score += 3.0; // Première phrase très importante
    } else if position == total - 1 {
        score += 2.0; // Dernière phrase importante
    } else if position < total / 3 {
        score += 1.0; // Premier tiers
    }

    // 2. Longueur optimale (30-120 caractères)
    let len = sentence.len();
    if len >= 30 && len <= 120 {
        score += 2.0;
    } else if len > 120 && len <= 200 {
        score += 1.0;
    } else if len < 15 {
        score -= 1.0; // Trop court, probablement peu informatif
    }

    // 3. Ponctuation forte (questions, exclamations)
    if sentence.contains('?') {
        score += 2.5;
    }
    if sentence.contains('!') {
        score += 1.5;
    }

    // 4. Mots-clés importants (technique, action, problème)
    let keywords_high = [
        "erreur", "error", "problème", "solution", "important",
        "obligatoire", "nécessaire", "essentiel", "critique",
        "bug", "fix", "feature", "implémentation", "optimis"
    ];
    let keywords_medium = [
        "fonction", "code", "fichier", "variable", "résultat",
        "créer", "modifier", "ajouter", "utiliser", "exemple",
        "type", "struct", "impl", "trait", "module"
    ];

    let lower = sentence.to_lowercase();
    for keyword in keywords_high.iter() {
        if lower.contains(keyword) {
            score += 2.0;
        }
    }
    for keyword in keywords_medium.iter() {
        if lower.contains(keyword) {
            score += 1.0;
        }
    }

    // 5. Présence de nombres (données concrètes)
    if sentence.chars().any(|c| c.is_numeric()) {
        score += 1.5;
    }

    // 6. Noms propres et acronymes (majuscules en milieu de phrase)
    let words: Vec<&str> = sentence.split_whitespace().collect();
    let uppercase_words = words.iter()
        .skip(1) // Ignorer premier mot
        .filter(|w| w.chars().next().map_or(false, |c| c.is_uppercase()))
        .count();
    if uppercase_words > 0 {
        score += 1.0;
    }

    // 7. Structure de liste (-, *, numéros)
    let trimmed = sentence.trim();
    if trimmed.starts_with('-') || trimmed.starts_with('*') {
        score += 1.5;
    }
    if trimmed.chars().next().map_or(false, |c| c.is_numeric()) {
        score += 1.5;
    }

    // 8. Présence de chemins fichiers ou URLs (information technique)
    if sentence.contains('/') && (sentence.contains('.') || sentence.contains("src")) {
        score += 1.0;
    }

    // 9. Mots de négation ou contrastes (changement important)
    let contrast_words = ["mais", "cependant", "toutefois", "néanmoins", "par contre", "but", "however"];
    for word in contrast_words.iter() {
        if lower.contains(word) {
            score += 1.5;
        }
    }

    score
}

/// Génère un message de résumé pour un ensemble de messages
pub fn summarize_messages(messages: &[ChatMessage]) -> ChatMessage {
    let total_user = messages.iter().filter(|m| matches!(m.role, MessageRole::User)).count();
    let total_assistant = messages.iter().filter(|m| matches!(m.role, MessageRole::Assistant)).count();

    // Extraire les sujets principaux
    let mut topics = Vec::new();
    for msg in messages.iter() {
        if matches!(msg.role, MessageRole::User) {
            // Extraire les premiers mots significatifs
            let words: Vec<&str> = msg.content.split_whitespace().take(5).collect();
            if !words.is_empty() {
                topics.push(words.join(" "));
            }
        }
    }

    let summary = if topics.is_empty() {
        format!("[Historique précédent: {} échanges]", total_user)
    } else {
        format!(
            "[Historique précédent: {} échanges sur: {}]",
            total_user,
            topics.join(", ")
        )
    };

    ChatMessage {
        role: MessageRole::Assistant,
        content: summary,
    }
}

/// Supprime le contenu redondant des messages
pub fn remove_redundancy(messages: &[ChatMessage]) -> Vec<ChatMessage> {
    let mut optimized = Vec::new();

    for (i, msg) in messages.iter().enumerate() {
        let mut content = msg.content.clone();

        // Supprimer les formules de politesse courantes
        let politeness = [
            "Bien sûr, ",
            "Certainement, ",
            "D'accord, ",
            "Je comprends, ",
            "Merci pour votre question, ",
            "C'est une bonne question, ",
            "Volontiers, ",
        ];

        for phrase in &politeness {
            if content.starts_with(phrase) {
                content = content.replace(phrase, "");
            }
        }

        // Pour les messages assistant, supprimer les répétitions du contexte
        if matches!(msg.role, MessageRole::Assistant) && i > 0 {
            if let Some(prev_msg) = messages.get(i - 1) {
                // Si le message précédent est cité, le retirer
                let prev_words: Vec<&str> = prev_msg.content.split_whitespace().take(10).collect();
                let prev_start = prev_words.join(" ");

                if content.contains(&prev_start) && prev_start.len() > 20 {
                    content = content.replace(&prev_start, "[...]");
                }
            }
        }

        optimized.push(ChatMessage {
            role: msg.role.clone(),
            content: content.trim().to_string(),
        });
    }

    optimized
}

/// Génère le system prompt optimisé selon les options
pub fn generate_system_prompt(concise_mode: bool, use_search: bool) -> String {
    let mut prompt = String::from("Tu es un assistant IA intelligent et utile.");

    if concise_mode {
        prompt.push_str(" Réponds de manière concise et directe en 2-3 phrases maximum. Évite les répétitions et les formules de politesse inutiles.");
    } else {
        prompt.push_str(" Sois clair et précis dans tes réponses.");
    }

    if use_search {
        prompt.push_str(" Utilise la recherche web quand c'est pertinent pour donner des informations à jour.");
    }

    prompt
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_short_message() {
        let msg = ChatMessage {
            role: MessageRole::User,
            content: "Hello".to_string(),
        };
        let compressed = compress_message(&msg);
        assert_eq!(compressed.content, "Hello");
    }

    #[test]
    fn test_generate_system_prompt() {
        let prompt = generate_system_prompt(true, false);
        assert!(prompt.contains("concise"));
    }
}
