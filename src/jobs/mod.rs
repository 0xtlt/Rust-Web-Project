use std::str::FromStr;

pub mod print_hours;

pub struct RWPJob {
    pub name: String,
    pub cron_tab: cron::Schedule,
}

impl RWPJob {
    pub fn new(name: String, cron_tab: String) -> RWPJob {
        RWPJob {
            name,
            cron_tab: cron::Schedule::from_str(&cron_tab).unwrap(),
        }
    }

    // Cron system
    pub fn next_run(&self) -> chrono::DateTime<chrono::Utc> {
        self.cron_tab.upcoming(chrono::Utc).next().unwrap()
    }

    pub fn wait_time(&self) -> u64 {
        let next_run = self.next_run();
        let now = chrono::Utc::now();
        let duration = next_run - now;
        duration.num_seconds() as u64
    }
    // End of cron system

    // Debug/Log system
    pub fn log(&self, message: String) {
        println!("{}: {}", self.name, message);
    }
    // End of debug/log system
}
