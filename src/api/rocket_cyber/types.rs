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
