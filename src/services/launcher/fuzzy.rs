use nucleo::{Config, Nucleo, pattern::{CaseMatching, Normalization, Pattern}};

pub struct FuzzyMatcher {
    matcher: nucleo::Matcher,
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        Self {
            matcher: nucleo::Matcher::new(Config::DEFAULT),
        }
    }

    pub fn score(&mut self, pattern: &str, text: &str) -> Option<u32> {
        let pattern = Pattern::parse(pattern, CaseMatching::Ignore, Normalization::Smart);

        let mut buf = Vec::new();
        let haystack = nucleo::Utf32Str::new(text, &mut buf);

        pattern.score(haystack, &mut self.matcher)
    }

    pub fn rank<T, F>(&mut self, items: Vec<T>, pattern: &str, get_text: F) -> Vec<(T, u32)>
    where
        F: Fn(&T) -> &str,
    {
        let mut scored: Vec<(T, u32)> = items
            .into_iter()
            .filter_map(|item| {
                let text = get_text(&item);
                self.score(pattern, text).map(|score| (item, score))
            })
            .collect();

        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored
    }
}
