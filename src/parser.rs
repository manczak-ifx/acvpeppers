use serde::{Deserialize, Deserializer};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestVectorSet {
    pub vs_id: u64,
    pub algorithm: String,
    pub revision: String,
    pub test_groups: Vec<TestGroup>, 
}
#[derive(Debug)]
pub struct TestGroup {
    pub tg_id: u64,
    pub test_type: TestType,
    pub mctversion: Option<String>,     
    pub tests: Vec<TestCase>,          
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawTestGroup {
    pub tg_id: u64,
    pub test_type: TestType,
    pub mctversion: Option<String>,     
    pub tests: Vec<TestCase>,            
}

impl<'de> Deserialize<'de> for TestGroup {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut raw = RawTestGroup::deserialize(deserializer)?;
        if let Some(ref v) = raw.mctversion {
            for t in &mut raw.tests {
                t.mct_version = Some(v.clone());
            }
        }

        Ok(TestGroup {
            tg_id: raw.tg_id,
            test_type: raw.test_type,
            mctversion: raw.mctversion,
            tests: raw.tests,
        })
    }
}


#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCase {
    pub tc_id: u64,
    //for sha2
    pub len: Option<u32>,
    pub msg: Option<String>,
    //for AES
    pub key: Option<String>,
    pub ct: Option<String>,
    pub iv : Option<String>,
    pub plaintext: Option<String>,
    pub large_msg : Option<LargeMsg>,
    #[serde(skip_deserializing, skip_serializing, default)]
    pub mct_version: Option<String>,
    
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LargeMsg {
    pub content : String,
    pub content_length: u64,
    pub full_length : u128,
    pub expansion_technique: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum TestType {
    AFT,
    MCT,
    LDT,
}
