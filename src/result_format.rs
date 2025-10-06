use serde::Serialize;
use serde_json::{json, Value};

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestResultSet {
    pub vs_id: u64,                       
    pub algorithm: String,             
    #[serde(default)]
    pub revision: String,        
    pub test_groups: Vec<TestGroupResult>
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestGroupResult {
    pub tg_id: u64,                     
    pub tests: Vec<TestCaseResult>,    
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseResult {
    pub tc_id: u64,                       
    pub md: String,                      
}
