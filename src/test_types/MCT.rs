use crate::cryptography::CryptoOperation;
use crate::parser::TestCase;
use crate::result_format::TestCaseResult;

pub struct MCT;

impl crate::test_types::TestExecutor for MCT {
    fn execute_test(
        &self,
        crypto: &dyn CryptoOperation,
        tc: &TestCase,
    ) -> TestCaseResult {
        let flavor = tc
            .mct_version
            .as_deref()
            .unwrap_or("standard")
            .to_ascii_lowercase();

        if tc.msg.is_some() {
            // sha2
            run_hash_mct(crypto, tc, &flavor)
        } else if tc.key.is_some() && tc.plaintext.is_some() {
            // AES
            run_aes_mct_scaffold(crypto, tc, &flavor)
        } else {
           
            TestCaseResult {
                tc_id: tc.tc_id,
                md: String::new(),
            }
        }
    }
}


fn run_hash_mct(
    crypto: &dyn CryptoOperation,
    tc: &TestCase,
    flavor: &str,
) -> TestCaseResult {
    let outer = 1000usize;
    let mut cur = tc.clone();
    if cur.msg.is_none() {
        cur.msg = Some(String::new());
        cur.len = Some(0);
    } else if cur.len.is_none() {
        let start_bytes = hex::decode(cur.msg.as_ref().unwrap()).unwrap_or_default();
        cur.len = Some((start_bytes.len() * 8) as u32);
    }

    let mut outputs: Vec<String> = Vec::with_capacity(outer);

    for i in 0..outer {
        if i > 0 {
            match flavor {
                "alternate" => {
                    let next_bytes: Vec<u8> = if outputs.len() >= 3 {
                        let a = hex::decode(&outputs[outputs.len() - 3]).unwrap_or_default();
                        let b = hex::decode(&outputs[outputs.len() - 2]).unwrap_or_default();
                        let c = hex::decode(&outputs[outputs.len() - 1]).unwrap_or_default();
                        [a, b, c].concat()
                    } else {
                        hex::decode(outputs.last().unwrap()).unwrap_or_default()
                    };
                    cur.msg = Some(hex::encode_upper(&next_bytes));
                    cur.len = Some((next_bytes.len() * 8) as u32);
                }
                
                _ => {
                    let prev_bytes = hex::decode(outputs.last().unwrap()).unwrap_or_default();
                    cur.msg = Some(hex::encode_upper(&prev_bytes));
                    cur.len = Some((prev_bytes.len() * 8) as u32);
                }
            }
        }
        let md_hex = crypto.execute(&cur);
        outputs.push(md_hex);
    }

    TestCaseResult {
        tc_id: tc.tc_id,
        md: outputs.last().cloned().unwrap_or_default(),
    }
}


fn run_aes_mct_scaffold(
    crypto: &dyn CryptoOperation,
    tc: &TestCase,
    flavor: &str,
) -> TestCaseResult {

    let outer = 1000usize;
    let inner = 1000usize;

    let mut cur = tc.clone();

    let mut key = hex::decode(cur.key.as_ref().unwrap()).unwrap_or_default();
    let mut pt  = hex::decode(cur.plaintext.as_ref().unwrap()).unwrap_or_default();
    let mut last_cts: Vec<Vec<u8>> = Vec::new();

    for _i in 0..outer {
        let mut last_ct: Vec<u8> = Vec::new();

        for _j in 0..inner {
            cur.key = Some(hex::encode_upper(&key));
            let pt_for_this_iter = if flavor == "alternate" && last_cts.len() >= 3 {
                [last_cts[last_cts.len() - 3].as_slice(),
                 last_cts[last_cts.len() - 2].as_slice(),
                 last_cts[last_cts.len() - 1].as_slice()].concat()
            } else {
                pt.clone()
            };

            cur.plaintext = Some(hex::encode_upper(&pt_for_this_iter));
            let ct_hex = crypto.execute(&cur);
            last_ct = hex::decode(ct_hex).unwrap_or_default();

            // PT chaining for next inner iter
            pt = last_ct.clone();

            // Track recent CTs for "alternate" flavor
            last_cts.push(last_ct.clone());
            if last_cts.len() > 3 {
                last_cts.remove(0);
            }
        }
        //XOR
        if !last_ct.is_empty() && !key.is_empty() {
            for (i, kb) in key.iter_mut().enumerate() {
                *kb ^= last_ct[i % last_ct.len()];
            }
        }

       
    }

    TestCaseResult {
        tc_id: tc.tc_id,
        md: hex::encode_upper(&pt),
    }
}
