use std::str::FromStr;

pub mod print_minutes;

pub struct RWPJob {
    pub name: String,
    pub cron_tab: cron::Schedule,
    pub last_run: chrono::DateTime<chrono::Utc>,
    // If true, the function will be run at the next tick
    pub first_time: bool,
    // Async function
    pub job: fn(),
}

impl RWPJob {
    pub fn new(name: String, cron_tab: String, first_time: bool, job: fn()) -> RWPJob {
        RWPJob {
            name,
            cron_tab: cron::Schedule::from_str(&cron_tab).unwrap(),
            last_run: chrono::Utc::now(),
            first_time,
            job,
        }
    }

    // Cron system
    pub fn next_run(&self) -> chrono::DateTime<chrono::Utc> {
        self.cron_tab.after(&chrono::Utc::now()).next().unwrap()
    }

    pub fn wait_time_in_ms(&self) -> u64 {
        let next_run = self.next_run();
        let now = chrono::Utc::now();
        let duration = next_run - now;
        (duration.num_milliseconds() + 100) as u64
    }
    // End of cron system

    // Debug/Log system
    pub fn log(&self, message: String) {
        println!("{}: {}", self.name, message);
    }
    // End of debug/log system
}
