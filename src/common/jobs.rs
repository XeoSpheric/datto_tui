use crate::api::datto::types::JobResult;
use crate::app::JobViewRow;

/// Generates a list of JobViewRow enums based on the contents of a JobResult.
/// This determines how the job result detail view should be structured,
/// mapping components to headers and adding links for StdOut/StdErr if they exist.
///
/// # Arguments
/// * `job_result` - The JobResult containing component outcomes and output status.
///
/// # Returns
/// A vector of JobViewRow used by the UI to render the job detail page.
pub fn generate_job_rows(job_result: &JobResult) -> Vec<JobViewRow> {
    let mut rows = Vec::new();
    if let Some(components) = &job_result.component_results {
        for (idx, comp) in components.iter().enumerate() {
            rows.push(JobViewRow::ComponentHeader(idx));
            if comp.has_std_out == Some(true) {
                rows.push(JobViewRow::StdOutLink(idx));
            }
            if comp.has_std_err == Some(true) {
                rows.push(JobViewRow::StdErrLink(idx));
            }
        }
    }
    rows
}
