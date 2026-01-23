use crate::api::datto::types::JobResult;
use crate::app::JobViewRow;

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
