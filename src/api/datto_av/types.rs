use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AgentDetail {
    pub isolated: Option<bool>,
    pub unsupported_os: Option<bool>,
    pub has_edr_license: Option<bool>,
    pub has_av_license: Option<bool>,
    pub authorized_sort_key: Option<i32>,
    pub active_sort_key: Option<i32>,
    pub status: Option<String>,
    pub device_group_name: Option<String>,
    pub organization_id: Option<String>,
    pub alert_count: Option<String>,
    pub location_id: Option<String>,
    pub location_name: Option<String>,
    pub organization_name: Option<String>,

    // Using Value for objects that were empty in example or complex
    pub epp_data: Option<serde_json::Value>,
    pub rwd_info: Option<serde_json::Value>,

    pub metrics_collection: Option<String>,
    pub desktop_ui_enabled: Option<bool>,
    pub sync_status: Option<bool>,

    pub id: String,
    pub name: Option<String>,
    pub hostname: String,
    pub version: Option<String>,
    pub authorized: Option<bool>,
    pub marked_for_uninstall: Option<bool>,
    pub marked_for_update: Option<bool>,

    pub ip: Option<String>,
    #[serde(rename = "ipstring")]
    pub ip_string: Option<String>,

    pub os: Option<String>,
    pub os_windows: Option<bool>,
    pub os_osx: Option<bool>,
    pub os_linux: Option<bool>,
    pub os_other: Option<bool>,

    pub heartbeat: Option<String>,
    pub active: Option<bool>,
    pub monitored_job_fetched: Option<String>,

    #[serde(rename = "type")]
    pub type_field: Option<String>,

    pub vsa_id: Option<String>,
    pub device_id: Option<String>,
    pub device_group_id: Option<String>,

    pub datto_av_enabled: Option<bool>,
    pub dns_secure_enabled: Option<bool>,
    pub dns_secure_last_enabled: Option<String>,

    pub data: Option<serde_json::Value>,
    pub marked_for_update_on: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub id: String,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
    pub source_id: Option<String>,
    pub source_version_id: Option<String>,
    pub source_type: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub mitre_id: Option<String>,
    pub mitre_tactic: Option<String>,
    pub source_name: Option<String>,
    pub ip: Option<String>,
    pub search: Option<String>,
    pub hostname: Option<String>,
    pub item_id: Option<String>,
    pub host_id: Option<String>,
    pub host_scan_id: Option<String>,
    pub scan_id: Option<String>,
    pub batch_id: Option<String>,
    pub file_rep_id: Option<String>,
    pub signed: Option<bool>,
    pub managed: Option<bool>,
    pub created_on: Option<String>,
    pub event_time: Option<String>,
    pub data: Option<serde_json::Value>,
    pub signal: Option<bool>,
    pub archived: Option<bool>,
    pub extension_success: Option<bool>,
    pub av_ratio: Option<f64>,
    pub agent_id: Option<String>,
    pub tenant: Option<String>,
    pub created_date: Option<String>,
    pub target_group_id: Option<String>,
    pub target_group_name: Option<String>,
    pub device_id: Option<String>,
    pub vsa_id: Option<String>,
    pub rmm_site_id: Option<String>,
    pub rmm_account_id: Option<String>,
    pub organization_id: Option<String>,
    pub organization_name: Option<String>,
    pub suppression_rule_version_id: Option<String>,
    pub response_data: Option<String>,
}
