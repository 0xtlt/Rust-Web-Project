use super::RWPJob;

pub fn job_function() {
    println!("Hello, world, it's {}", chrono::Utc::now());
}

pub fn job_print_minutes() -> Vec<RWPJob> {
    let ok = RWPJob::new(
        "print_minutes".to_string(),
        "0 * * * * * *".to_string(),
        true,
        job_function,
    );

    vec![ok]
}
