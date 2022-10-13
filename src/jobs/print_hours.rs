use super::RWPJob;

pub fn job_print_hours() -> RWPJob {
    RWPJob::new("print_hours".to_string(), "0 * * * *".to_string())
}
