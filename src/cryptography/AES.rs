use aes::{Aes128, Block, BlockEncrypt, NewBlockCipher};
use crate::cryptography::CryptoOperation;
use crate::parser::TestCase;

pub struct AES {
    pub key: Vec<u8>, 
}

impl CryptoOperation for AES {
    fn execute(&self, test_case: &TestCase) -> String {
        let mut input = hex::decode(test_case.msg.as_ref().expect("Missing 'msg'"))
            .expect("Invalid hex in 'msg'");
        let cipher = Aes128::new_from_slice(&self.key).expect("Invalid AES key");
        let mut block = Block::clone_from_slice(&input);
        cipher.encrypt_block(&mut block);
        hex::encode_upper(block)
    }
}
