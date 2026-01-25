use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Incident {
    pub id: i32,
    pub title: String,
    pub status: String,
    pub account_id: i32,
    pub account_name: String,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct IncidentsResponse {
    pub total_count: i32,
    pub data_count: i32,
    pub data: Vec<Incident>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub customer_id: i32,
    pub customer_name: Option<String>,
    pub hostname: String,
    pub ipv4_address: String,
    pub mac_address: String,
    pub created_at: String,
    pub platform: String,
    pub family: String,
    pub version: String,
    pub edition: String,
    pub architecture: String,
    pub build: String,
    pub release: String,
    pub operating_system: Option<String>,
    pub account_path: String,
    pub agent_version: String,
    pub connectivity: String,
    pub last_connected_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentsResponse {
    pub total_count: i32,
    pub current_page: i32,
    pub total_pages: i32,
    pub data_count: i32,
    pub data: Vec<Agent>,
}
