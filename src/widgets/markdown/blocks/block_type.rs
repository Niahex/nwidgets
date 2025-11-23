/// Types de blocs supportés dans l'éditeur markdown
#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    /// Paragraphe simple
    Paragraph,

    /// Titre (niveau 1-5)
    Heading(u8),

    /// Liste à puces
    BulletList,

    /// Liste numérotée (numéro de départ)
    NumberedList(u32),

    /// Toggle simple
    Toggle,

    /// Toggle avec titre (niveau 1-5)
    ToggleHeading(u8),

    /// Divider horizontal
    Divider,

    /// Bloc de code (langage optionnel)
    CodeBlock(Option<String>),
}

impl BlockType {
    /// Retourne le nom du type de bloc
    pub fn name(&self) -> &str {
        match self {
            BlockType::Paragraph => "paragraph",
            BlockType::Heading(_) => "heading",
            BlockType::BulletList => "bullet_list",
            BlockType::NumberedList(_) => "numbered_list",
            BlockType::Toggle => "toggle",
            BlockType::ToggleHeading(_) => "toggle_heading",
            BlockType::Divider => "divider",
            BlockType::CodeBlock(_) => "code_block",
        }
    }

    /// Détecte le type de bloc à partir d'une ligne de texte
    pub fn from_line(line: &str) -> Self {
        let trimmed = line.trim_start();

        // Divider
        if trimmed == "---" {
            return BlockType::Divider;
        }

        // Toggles avec titres
        if trimmed.starts_with(">##### ") {
            return BlockType::ToggleHeading(5);
        } else if trimmed.starts_with(">#### ") {
            return BlockType::ToggleHeading(4);
        } else if trimmed.starts_with(">### ") {
            return BlockType::ToggleHeading(3);
        } else if trimmed.starts_with(">## ") {
            return BlockType::ToggleHeading(2);
        } else if trimmed.starts_with("># ") {
            return BlockType::ToggleHeading(1);
        }

        // Toggle simple
        if trimmed.starts_with("> ") {
            return BlockType::Toggle;
        }

        // Titres
        if trimmed.starts_with("##### ") {
            return BlockType::Heading(5);
        } else if trimmed.starts_with("#### ") {
            return BlockType::Heading(4);
        } else if trimmed.starts_with("### ") {
            return BlockType::Heading(3);
        } else if trimmed.starts_with("## ") {
            return BlockType::Heading(2);
        } else if trimmed.starts_with("# ") {
            return BlockType::Heading(1);
        }

        // Liste numérotée
        if let Some(num) = Self::parse_numbered_list_start(trimmed) {
            return BlockType::NumberedList(num);
        }

        // Liste à puces
        if trimmed.starts_with("- ") {
            return BlockType::BulletList;
        }

        // Bloc de code
        if trimmed.starts_with("```") {
            let lang = if trimmed.len() > 3 {
                Some(trimmed[3..].trim().to_string())
            } else {
                None
            };
            return BlockType::CodeBlock(lang);
        }

        // Par défaut : paragraphe
        BlockType::Paragraph
    }

    /// Parse le début d'une liste numérotée
    fn parse_numbered_list_start(line: &str) -> Option<u32> {
        let chars: Vec<char> = line.chars().collect();
        let mut num_str = String::new();

        for (i, &ch) in chars.iter().enumerate() {
            if ch.is_ascii_digit() {
                num_str.push(ch);
            } else if ch == '.' && !num_str.is_empty() {
                if i + 1 < chars.len() && chars[i + 1] == ' ' {
                    return num_str.parse().ok();
                } else {
                    return None;
                }
            } else {
                break;
            }
        }
        None
    }

    /// Extrait le contenu sans le marqueur
    pub fn strip_marker<'a>(&self, line: &'a str) -> &'a str {
        let trimmed = line.trim_start();

        match self {
            BlockType::Heading(level) => {
                let marker_len = *level as usize + 1; // "# ", "## ", etc.
                if trimmed.len() > marker_len {
                    trimmed[marker_len..].trim_start()
                } else {
                    ""
                }
            }
            BlockType::BulletList => {
                if trimmed.len() > 2 {
                    &trimmed[2..]
                } else {
                    ""
                }
            }
            BlockType::NumberedList(_) => {
                // Trouver la position après "N. "
                if let Some(pos) = trimmed.find(". ") {
                    &trimmed[pos + 2..]
                } else {
                    trimmed
                }
            }
            BlockType::Toggle => {
                if trimmed.len() > 2 {
                    &trimmed[2..]
                } else {
                    ""
                }
            }
            BlockType::ToggleHeading(level) => {
                let marker_len = 1 + *level as usize + 1; // ">", "#", " "
                if trimmed.len() > marker_len {
                    trimmed[marker_len..].trim_start()
                } else {
                    ""
                }
            }
            BlockType::CodeBlock(_) => {
                if trimmed.starts_with("```") {
                    &trimmed[3..]
                } else {
                    trimmed
                }
            }
            BlockType::Divider | BlockType::Paragraph => trimmed,
        }
    }
}
