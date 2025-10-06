use std::collections::HashMap;
use std::sync::Arc;
use crate::cryptography::{ CryptoOperation, aes_ecb::AesEcbOp};

pub struct CryptoRegistry {
    registry: HashMap<String, Arc<dyn CryptoOperation>>,
}

impl CryptoRegistry {
    pub fn new() -> Self {
        Self {
            registry: HashMap::new(),
        }
    }
    pub fn register(&mut self, name: &str, crypto: Arc<dyn CryptoOperation>) {
        self.registry.insert(name.to_string(), crypto);
    }
    pub fn resolve(&self, name: &str) -> Option<Arc<dyn CryptoOperation>> {
        self.registry.get(name).cloned()
    }
}


pub fn initialize_crypto_registry() -> CryptoRegistry {
    let mut registry = CryptoRegistry::new();
    registry.register(
        "SHA2-256",
        Arc::new(crate::cryptography::sha2_alg::SHA2 { algorithm: "SHA2-256".to_string() }),
    );
    registry.register(
        "SHA2-512",
        Arc::new(crate::cryptography::sha2_alg::SHA2 { algorithm: "SHA2-512".to_string() }),
    );
    registry.register("ACVP-AES-ECB".into(), Arc::new(AesEcbOp));

    registry
}
