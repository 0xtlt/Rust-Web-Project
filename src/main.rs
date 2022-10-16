#[macro_use]
extern crate lazy_static;
pub mod fairings;
pub mod guards;
pub mod handlers;
pub mod jobs;
mod prisma;

use async_once::AsyncOnce;
use prisma::PrismaClient;
use serde::Deserialize;
use tokio::fs;

use crate::jobs::print_minutes::job_print_minutes;

#[derive(Deserialize, Debug)]
struct RWPConfigFile {
    // by default: none, * for all and a comma separated list of domains for specific ones
    pub config: RWPOptions,
}

#[derive(Deserialize, Debug)]
struct RWPOptions {
    pub http_server: Option<RWPHttpServer>,
    pub database: Option<RWPDatabase>,
}

#[derive(Deserialize, Debug)]
struct RWPHttpServer {
    // by default: none, * for all and a comma separated list of domains for specific ones
    pub cors_mode: Option<String>,
}

#[derive(Deserialize, Debug)]
struct RWPDatabase {
    // by default: mysql
    pub kind: String,
    pub username: String,
    pub password: String,
    pub host: String,
    // by default: 3306
    pub port: Option<u16>,
    pub database: String,
    pub more_params: Option<String>,
}

struct HttpServerStates<'a> {
    database: &'a Option<PrismaClient>,
}

lazy_static! {
    static ref RWP_OPTIONS: AsyncOnce<RWPConfigFile> = AsyncOnce::new(async {
        toml::from_str(
            &fs::read_to_string("RWP.toml")
                .await
                .expect("Could not read rwp-options.json"),
        )
        .unwrap()
    });
    static ref DATABASE: AsyncOnce<Option<PrismaClient>> = AsyncOnce::new(async {
        if let Some(config_database) = &RWP_OPTIONS.get().await.config.database {
            let tmp_empty_string = "".to_string();

            println!("Loading database...");
            let database_url = format!(
                "{}://{}:{}@{}:{}/{}?{}",
                config_database.kind,
                config_database.username,
                config_database.password,
                config_database.host,
                config_database.port.unwrap_or(3306),
                config_database.database,
                config_database
                    .more_params
                    .as_ref()
                    .unwrap_or(&tmp_empty_string)
            );

            println!("Database loaded.");
            Some(prisma::new_client_with_url(&database_url).await.unwrap())
        } else {
            None
        }
    });
}

#[tokio::main]
async fn main() {
    println!("Starting RWP server...");

    let jobs: Vec<jobs::RWPJob> = vec![job_print_minutes()].into_iter().flatten().collect();

    for job in jobs {
        println!("{}: {}", job.name, job.wait_time_in_ms());

        // Spawn a new task for each job
        tokio::spawn(async move {
            let mut is_first_time = true;
            loop {
                // Wait for the job to run
                if is_first_time {
                    is_first_time = false;
                } else {
                    tokio::time::sleep(std::time::Duration::from_millis(job.wait_time_in_ms()))
                        .await;
                }

                // Run the job
                job.log("Running job".to_string());
                (job.job)();
            }
        });
    }

    if let Some(config_http_server) = &RWP_OPTIONS.get().await.config.http_server {
        println!("Starting web server...");
        let http_rocket = {
            let mut http_rocket = rocket::build();

            // If you want to use the CORS fairing, uncomment the following line:
            if let Some(cors_mode) = &config_http_server.cors_mode {
                http_rocket = http_rocket.attach(fairings::cors::Cors {
                    mode: cors_mode.to_string(),
                });
            }

            http_rocket.manage(HttpServerStates {
                database: DATABASE.get().await,
            })
        }
        .mount("/", rocket::routes![]);

        let _ = http_rocket.launch().await.unwrap();
    }
}
