use std::collections::HashMap;
use crate::{cryptography::{ CryptoOperation}};

pub struct CryptoRegistry {
    registry: HashMap<String, Box<dyn CryptoOperation>>,
}

impl CryptoRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }
    pub fn register(&mut self, name: &str, crypto: Box<dyn CryptoOperation>) {
        self.registry.insert(name.to_string(), crypto);
    }
    pub fn resolve(&self, name: &str) -> Option<&Box<dyn CryptoOperation>> {
        self.registry.get(name)
    }
}


pub fn initialize_crypto_registry() -> CryptoRegistry {
    let mut registry = CryptoRegistry::new();
    registry.register(
        "SHA2-256",
        Box::new(crate::cryptography::sha2_alg::SHA2 { algorithm: "SHA2-256".to_string() }),
    );
    registry.register(
        "SHA2-512",
        Box::new(crate::cryptography::sha2_alg::SHA2 { algorithm: "SHA2-512".to_string() }),
    );

    registry
}
