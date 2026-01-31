use numbat::module_importer::BuiltinModuleImporter;
use numbat::Context;

pub struct CalculatorService {
    context: Context,
}

impl CalculatorService {
    pub fn new() -> Self {
        let importer = BuiltinModuleImporter::default();
        let mut context = Context::new(importer);

        let _ = context.interpret("", 1);

        Self { context }
    }

    pub fn evaluate(&mut self, expression: &str) -> Result<String, String> {
        let result = self.context.interpret(expression, 1);

        match result {
            Ok((statements, result)) => {
                if let Some(value) = result.to_markup(statements.last(), true, true) {
                    Ok(value.to_string())
                } else {
                    Err("No result".to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
