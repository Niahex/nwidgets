use numbat::module_importer::BuiltinModuleImporter;
use numbat::resolver::CodeSource;
use numbat::Context;

pub struct CalculatorService {
    context: Context,
}

impl CalculatorService {
    pub fn new() -> Self {
        let importer = BuiltinModuleImporter::default();
        let mut context = Context::new(importer);

        let _ = context.interpret("use prelude", CodeSource::Internal);

        Self { context }
    }

    pub fn evaluate(&mut self, expression: &str) -> Result<String, String> {
        let result = self.context.interpret(expression, CodeSource::Text);

        match result {
            Ok((statements, result)) => {
                let markup = result.to_markup(
                    statements.last(),
                    self.context.dimension_registry(),
                    true,
                    true,
                );
                Ok(markup.to_string())
            }
            Err(e) => Err(e.to_string()),
        }
    }
}
