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
    #[serde(skip, default)]
    pub variables: Option<Vec<SiteVariable>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiteVariable {
    pub id: i32,
    pub name: String,
    pub value: String,
    pub masked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SiteVariablesResponse {
    pub page_details: PageDetails,
    pub variables: Vec<SiteVariable>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreateVariableRequest {
    pub name: String,
    pub value: String,
    pub masked: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVariableRequest {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSiteRequest {
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub on_demand: Option<bool>,
    pub splashtop_auto_install: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatchManagement {
    pub patch_status: Option<String>,
    pub patches_approved_pending: Option<i32>,
    pub patches_not_approved: Option<i32>,
    pub patches_installed: Option<i32>,
    pub patches_pending: Option<i32>, // Adding this as user mentioned "Patches Pending" in example, though distinct from ApprovedPending? Or maybe they mean the same. The user said "patchesApprovedPending, patchesNotApproved, and PatchesInstalled" but the example string had "Patches Pending: 1". Let's assume "Approved Pending" maps to patchesApprovedPending and "Patches Pending" might be another field or a mis-speak.
                                      // Wait, the user said: "Patch Status: Approved Pending | Patches Installed: 1 | Patches Pending: 1 | Patches Not Approved: 0"
                                      // And asked to add: "patchesApprovedPending, patchesNotApproved, and PatchesInstalled".
                                      // It seems "Approved Pending" is the Status Text.
                                      // "Patches Pending" in the example likely corresponds to `patchesApprovedPending` count? Or is there a generic `patchesPending`?
                                      // Let's look at common RMM APIs. Usually there's `patchesApprovedPending` and `patchesPending` (total?).
                                      // I'll add `patchesPending` just in case, or map `patchesApprovedPending` to "Patches Pending" in the UI if that's what they meant.
                                      // Re-reading: "patchesApprovedPending, patchesNotApproved, and PatchesInstalled".
                                      // I will add these 3 specific fields.
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Udf {
    pub udf1: Option<String>,
    pub udf2: Option<String>,
    pub udf3: Option<String>,
    pub udf4: Option<String>,
    pub udf5: Option<String>,
    pub udf6: Option<String>,
    pub udf7: Option<String>,
    pub udf8: Option<String>,
    pub udf9: Option<String>,
    pub udf10: Option<String>,
    pub udf11: Option<String>,
    pub udf12: Option<String>,
    pub udf13: Option<String>,
    pub udf14: Option<String>,
    pub udf15: Option<String>,
    pub udf16: Option<String>,
    pub udf17: Option<String>,
    pub udf18: Option<String>,
    pub udf19: Option<String>,
    pub udf20: Option<String>,
    pub udf21: Option<String>,
    pub udf22: Option<String>,
    pub udf23: Option<String>,
    pub udf24: Option<String>,
    pub udf25: Option<String>,
    pub udf26: Option<String>,
    pub udf27: Option<String>,
    pub udf28: Option<String>,
    pub udf29: Option<String>,
    pub udf30: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Antivirus {
    pub antivirus_product: Option<String>,
    pub antivirus_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeviceType {
    pub category: Option<String>,
    #[serde(rename = "type")]
    pub type_field: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: i32,
    pub uid: String,
    pub site_id: i32,
    pub site_uid: String,
    pub site_name: Option<String>,
    pub hostname: String,
    pub description: Option<String>,
    pub online: bool,
    #[serde(rename = "lastSeen")]
    // Note: User provided example string "2026-01-17T19:38:38.330Z" but also mentioned "number it gives right now (example Last Seen: 1768448871000 )"
    // The previous implementation used i64 (timestamp). The user request says "Last Seen: 1768448871000" which is a timestamp.
    // However, the JSON object example shows "lastSeen": "2026-01-17T19:38:38.330Z".
    // This suggests the API might return either depending on endpoint or version, OR they want us to handle the timestamp they see currently via converting it.
    // Given the previous code used `i64`, let's stick to `serde_json::Value` or try to support both, OR assuming the initial `i64` was correct for the current endpoint.
    // BUT the user says "the number it gives right now (example Last Seen: 1768448871000 )".
    // So let's keep it as i64 or Option<serde_json::Value> to be safe, but let's try strict typing if possible.
    // If the API returns a number, we keep i64. If it returns a string, we need String.
    // Let's assume it is still a number (i64) based on "the number it gives right now".
    pub last_seen: Option<serde_json::Value>,
    pub operating_system: Option<String>,
    pub patch_management: Option<PatchManagement>,

    // New Fields
    pub device_type: Option<DeviceType>,
    pub int_ip_address: Option<String>,
    pub ext_ip_address: Option<String>,
    pub last_logged_in_user: Option<String>,
    pub domain: Option<String>,
    pub display_version: Option<String>,
    #[serde(rename = "a64Bit")]
    pub a64_bit: Option<bool>,
    pub reboot_required: Option<bool>,

    // Dates/Timestamps
    // Again, user says "Last Seen: 1768448871000" (number), but JSON example says ISO string.
    // Providing generic Value or trying to deserialize gracefully is best.
    // Let's try to use i64 for now if that is what was observed, but for new fields use Value to inspect.
    pub last_reboot: Option<serde_json::Value>,
    pub last_audit_date: Option<serde_json::Value>,
    pub creation_date: Option<serde_json::Value>,
    pub warranty_date: Option<String>, // Example says "string"

    pub udf: Option<Udf>,
    pub antivirus: Option<Antivirus>,

    pub snmp_enabled: Option<bool>,
    pub device_class: Option<String>,
    pub portal_url: Option<String>,
    pub web_remote_url: Option<String>,
    pub network_probe: Option<bool>,
    pub onboarded_via_network_monitor: Option<bool>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivitySite {
    pub id: i32,
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityUser {
    pub id: i32,
    pub user_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLog {
    pub id: Option<String>,
    pub entity: Option<String>,
    pub category: Option<String>,
    pub action: Option<String>,
    pub date: Option<f64>,
    pub site: Option<ActivitySite>,
    pub device_id: Option<i32>,
    pub hostname: Option<String>,
    pub user: Option<ActivityUser>,
    pub details: Option<String>,
    pub has_std_out: Option<bool>,
    pub has_std_err: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLogsResponse {
    pub page_details: Option<PageDetails>,
    pub activities: Vec<ActivityLog>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentResult {
    pub component_uid: Option<String>,
    pub component_name: Option<String>,
    pub component_status: Option<String>,
    pub number_of_warnings: Option<i32>,
    pub has_std_out: Option<bool>,
    pub has_std_err: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobResult {
    pub job_uid: Option<String>,
    pub device_uid: Option<String>,
    pub ran_on: Option<serde_json::Value>, // Changed to Value to accept number or string
    pub job_deployment_status: Option<String>,
    pub component_results: Option<Vec<ComponentResult>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct JobStdOutput {
    pub component_uid: Option<String>,
    pub component_name: Option<String>,
    pub std_data: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlertMonitorInfo {
    pub sends_emails: Option<bool>,
    pub creates_ticket: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlertContext {
    #[serde(rename = "@class")]
    pub class: Option<String>,
    pub package_name: Option<String>,
    pub action_type: Option<String>,
    pub prev_version: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlertSourceInfo {
    pub device_uid: Option<String>,
    pub device_name: Option<String>,
    pub site_uid: Option<String>,
    pub site_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AlertResponseAction {
    pub action_time: Option<serde_json::Value>,
    pub action_type: Option<String>,
    pub description: Option<String>,
    pub action_reference: Option<String>,
    pub action_reference_int: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub alert_uid: Option<String>,
    pub priority: Option<String>,
    pub diagnostics: Option<String>,
    pub resolved: Option<bool>,
    pub resolved_by: Option<String>,
    pub resolved_on: Option<serde_json::Value>,
    pub muted: Option<bool>,
    pub ticket_number: Option<String>,
    pub timestamp: Option<serde_json::Value>,
    pub alert_monitor_info: Option<AlertMonitorInfo>,
    pub alert_context: Option<AlertContext>,
    pub alert_source_info: Option<AlertSourceInfo>,
    pub response_actions: Option<Vec<AlertResponseAction>>,
    pub autoresolve_mins: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenAlertsResponse {
    pub page_details: PageDetails,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentVariable {
    pub name: String,
    pub default_val: Option<String>,
    #[serde(rename = "type")]
    pub variable_type: Option<String>, // "Selection", "String", "Date", "Boolean"
    pub direction: Option<bool>, // true = input?
    pub description: Option<String>,
    pub variables_idx: Option<i32>,
    // For selection types, we might expect options, but the user example didn't provide them.
    // We'll see if we get them in a real response or if we need to parse them from somewhere else.
    // Sometimes selection options are comma separated in description or something, but usually a list.
    // I'll add a generic field just in case, but rely on user input if not found.
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Component {
    pub id: i32,
    pub credentials_required: Option<bool>,
    pub uid: String,
    pub name: String,
    pub description: Option<String>,
    pub category_code: Option<String>,
    pub variables: Option<Vec<ComponentVariable>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentsResponse {
    pub page_details: PageDetails,
    pub components: Vec<Component>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuickJobVariable {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuickJobComponent {
    pub component_uid: String,
    pub variables: Vec<QuickJobVariable>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuickJobRequest {
    pub job_name: String,
    pub job_component: QuickJobComponent,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuickJobInfo {
    pub id: i64,
    pub date_created: Option<serde_json::Value>,
    pub name: Option<String>,
    pub uid: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuickJobResponseVariable {
    pub name: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuickJobResponseComponent {
    pub uid: Option<String>,
    pub name: Option<String>,
    pub variables: Option<Vec<QuickJobResponseVariable>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QuickJobResponse {
    pub job_components: Option<Vec<QuickJobResponseComponent>>,
    pub job: Option<QuickJobInfo>,
}
