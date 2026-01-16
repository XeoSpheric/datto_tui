use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub expires_in: Option<i32>,
    pub token_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PageDetails {
    pub count: i32,
    pub total_count: Option<i32>,
    pub prev_page_url: Option<String>,
    pub next_page_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProxySettings {
    pub host: Option<String>,
    pub password: Option<String>,
    pub port: Option<i32>,
    pub type_field: Option<String>, // "type" is a reserved keyword
    pub username: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevicesStatus {
    pub number_of_devices: i32,
    pub number_of_online_devices: i32,
    pub number_of_offline_devices: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Site {
    pub id: i32,
    pub uid: String,
    pub account_uid: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub on_demand: Option<bool>,
    pub splashtop_auto_install: Option<bool>,
    pub proxy_settings: Option<ProxySettings>,
    pub devices_status: Option<DevicesStatus>,
    pub autotask_company_name: Option<String>,
    pub autotask_company_id: Option<String>,
    pub portal_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub uid: String,
    pub hostname: String,
    pub description: Option<String>,
    pub online: bool,
    #[serde(rename = "lastSeen")]
    pub last_seen: Option<i64>,
    #[serde(rename = "operatingSystem")]
    pub operating_system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DevicesResponse {
    pub page_details: PageDetails,
    pub devices: Vec<Device>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SitesResponse {
    pub page_details: PageDetails,
    pub sites: Vec<Site>,
}
