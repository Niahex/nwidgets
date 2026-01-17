use numbat::{module_importer::BuiltinModuleImporter, Context, InterpreterResult};

pub struct Calculator {
    context: Context,
}

impl Default for Calculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Calculator {
    pub fn new() -> Self {
        let importer = BuiltinModuleImporter::default();
        let context = Context::new(importer);

        Self { context }
    }

    pub fn evaluate(&mut self, expression: &str) -> Option<String> {
        // Remove the "=" prefix
        let expr = expression.strip_prefix('=').unwrap_or(expression).trim();

        if expr.is_empty() {
            return None;
        }

        match self
            .context
            .interpret(expr, numbat::resolver::CodeSource::Text)
        {
            Ok((_, InterpreterResult::Value(value))) => Some(format!("{value}")),
            Ok((_, InterpreterResult::Continue)) => None,
            Err(_) => None,
        }
    }
}

pub fn is_calculator_query(query: &str) -> bool {
    query.trim().starts_with('=')
}
