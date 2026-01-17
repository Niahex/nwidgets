use crate::widgets::launcher::state::ApplicationInfo;
use nucleo::{Config, Nucleo, Utf32String};

pub struct FuzzyMatcher {
    matcher: Nucleo<usize>,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    pub fn new() -> Self {
        let config = Config::DEFAULT;
        let matcher = Nucleo::new(config, std::sync::Arc::new(|| {}), None, 1);
        Self { matcher }
    }

    pub fn set_candidates(&mut self, apps: &[ApplicationInfo]) {
        // Clear previous items and reset
        self.matcher.restart(true);

        let injector = self.matcher.injector();
        for (i, app) in apps.iter().enumerate() {
            let name = Utf32String::from(app.name.as_str());
            let _ = injector.push(i, |cols| cols[0] = name.clone());
        }
    }

    pub fn search(&mut self, query: &str) -> Vec<usize> {
        if query.is_empty() {
            // Return all items if query is empty (limited by caller usually, but here we return all indices)
            // Ideally we'd know the count, but we can't easily get it from Nucleo without a snapshot.
            // Since the caller (main.rs) has the apps vector, they handle the "empty query" case 
            // usually by not calling search or taking all. 
            // But if they call search with empty string, Nucleo pattern matching might return everything if configured?
            // Let's rely on the pattern.
        }

        // Parse the pattern and update
        self.matcher.pattern.reparse(
            0,
            query,
            nucleo::pattern::CaseMatching::Ignore,
            nucleo::pattern::Normalization::Smart,
            false,
        );

        // Wait a bit for matching to complete (running in background threads managed by Nucleo)
        self.matcher.tick(10);

        // Get the results
        let snapshot = self.matcher.snapshot();
        let mut results: Vec<usize> = Vec::new();

        for i in 0..snapshot.matched_item_count() {
            if let Some(item) = snapshot.get_matched_item(i) {
                results.push(*item.data);
            }
        }

        results
    }
}
