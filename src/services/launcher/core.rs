use crate::components::SearchResult;
use crate::services::launcher::{
    applications, calculator, fuzzy::FuzzyMatcher, process, state::ApplicationInfo,
};
use applications::load_from_cache;
use calculator::{is_calculator_query, Calculator};
use parking_lot::RwLock;
use process::{get_running_processes, is_process_query, ProcessInfo};
use std::sync::Arc;

pub enum SearchResultType {
    Application(usize),
    Calculation(String),
    Process(ProcessInfo),
    Clipboard(String),
}

pub struct LauncherCore {
    pub applications: Arc<RwLock<Vec<ApplicationInfo>>>,
    pub fuzzy_matcher: FuzzyMatcher,
    pub calculator: Option<Calculator>,
}

impl LauncherCore {
    pub fn new() -> Self {
        Self {
            applications: Arc::new(RwLock::new(Vec::new())),
            fuzzy_matcher: FuzzyMatcher::new(),
            calculator: Some(Calculator::new()),
        }
    }

    pub fn load_from_cache(&mut self) {
        if let Some(apps) = load_from_cache() {
            *self.applications.write() = apps;
            self.fuzzy_matcher.set_candidates(&self.applications.read());
        }
    }

    pub fn search(&mut self, query: &str, clipboard_history: Vec<String>) -> Vec<SearchResultType> {
        let mut results = Vec::new();

        if query.starts_with("clip") {
            let search_term = if query.len() > 4 {
                query.strip_prefix("clip").unwrap_or("").to_lowercase()
            } else {
                String::new()
            };

            for entry in clipboard_history {
                if search_term.is_empty() || entry.to_lowercase().contains(&search_term) {
                    results.push(SearchResultType::Clipboard(entry));
                }
            }
        } else if is_process_query(query) {
            let processes = get_running_processes();
            if query == "ps" {
                for process in processes {
                    results.push(SearchResultType::Process(process));
                }
            } else if query.starts_with("ps") && query.len() > 2 {
                let search_term = query.strip_prefix("ps").unwrap_or("").to_lowercase();
                for process in processes {
                    if process.name.to_lowercase().contains(&search_term) {
                        results.push(SearchResultType::Process(process));
                    }
                }
            }
        } else if is_calculator_query(query) {
            if let Some(calculator) = &mut self.calculator {
                if let Some(result) = calculator.evaluate(query) {
                    results.push(SearchResultType::Calculation(result));
                }
            } else {
                results.push(SearchResultType::Calculation(
                    "Initializing calculator...".to_string(),
                ));
            }
        } else {
            let apps = self.applications.read();
            let app_indices = if query.is_empty() {
                (0..apps.len()).collect()
            } else {
                self.fuzzy_matcher.search(query)
            };

            for index in app_indices.into_iter().take(50) {
                results.push(SearchResultType::Application(index));
            }
        }

        results
    }

    pub fn convert_to_display_results(
        &self,
        internal_results: &[SearchResultType],
    ) -> Vec<SearchResult> {
        let apps = self.applications.read();
        internal_results
            .iter()
            .map(|result| match result {
                SearchResultType::Application(index) => {
                    if let Some(app) = apps.get(*index) {
                        SearchResult::Application(app.clone())
                    } else {
                        SearchResult::Application(ApplicationInfo {
                            name: "Invalid app".to_string(),
                            name_lower: "invalid app".to_string(),
                            exec: "".to_string(),
                            icon: None,
                            icon_path: None,
                        })
                    }
                }
                SearchResultType::Calculation(calc) => SearchResult::Calculation(calc.clone()),
                SearchResultType::Process(proc) => SearchResult::Process(proc.clone()),
                SearchResultType::Clipboard(content) => SearchResult::Clipboard(content.clone()),
            })
            .collect()
    }
}

impl Default for LauncherCore {
    fn default() -> Self {
        Self::new()
    }
}
