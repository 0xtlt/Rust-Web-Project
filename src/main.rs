pub mod fairings;
pub mod guards;
pub mod handlers;
pub mod jobs;
mod prisma;

use prisma::PrismaClient;
use serde::Deserialize;
use tokio::fs;

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

struct HttpServerStates {
    database: Option<PrismaClient>,
}

#[tokio::main]
async fn main() {
    let rwp_options: RWPConfigFile = toml::from_str(
        &fs::read_to_string("RWP.toml")
            .await
            .expect("Could not read rwp-options.json"),
    )
    .unwrap();

    println!("Starting RWP server...");
    println!("CORS mode: {:?}", rwp_options);

    let mut database: Option<PrismaClient> = None;
    if let Some(config_database) = rwp_options.config.database {
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
                .unwrap_or_else(|| "".to_string())
        );

        database = Some(prisma::new_client_with_url(&database_url).await.unwrap());
        println!("Database loaded.");
    }

    if let Some(config_http_server) = rwp_options.config.http_server {
        println!("Starting web server...");
        let http_rocket = {
            let mut http_rocket = rocket::build();

            // If you want to use the CORS fairing, uncomment the following line:
            if let Some(cors_mode) = config_http_server.cors_mode {
                http_rocket = http_rocket.attach(fairings::cors::Cors { mode: cors_mode });
            }

            http_rocket.manage(HttpServerStates { database })
        }
        .mount("/", rocket::routes![]);

        let _ = http_rocket.launch().await.unwrap();
    }
}
