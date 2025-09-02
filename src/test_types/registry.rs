use crate::test_types::{TestExecutor, MCT::MCT};
use std::collections::HashMap;
use crate::test_types::AFT::AFT;

pub struct TestTypeRegistry {
    registry: HashMap<String, Box<dyn TestExecutor>>,
}

impl TestTypeRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }
    pub fn register(&mut self, name: &str, executor: Box<dyn TestExecutor>) {
        self.registry.insert(name.to_string(), executor);
    }
    pub fn resolve(&self, name: &str) -> Option<&Box<dyn TestExecutor>> {
        self.registry.get(name)
    }
}
pub fn initialize_test_type_registry() -> TestTypeRegistry {
    let mut registry = TestTypeRegistry::new();
    registry.register("AFT", Box::new(AFT));
     registry.register("MCT", Box::new(MCT));
    registry
}