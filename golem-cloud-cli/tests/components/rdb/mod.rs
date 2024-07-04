pub mod docker_postgres;
pub mod sqlite;

use clap::Args;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tracing::{debug, error, info};

pub trait Rdb {
    fn info(&self) -> DbInfo;
    fn kill(&self);
}

#[derive(Debug)]
pub enum DbInfo {
    Sqlite(PathBuf),
    Postgres(PostgresInfo),
}

impl DbInfo {
    pub fn env(&self, app_name: &str) -> HashMap<String, String> {
        match self {
            DbInfo::Postgres(pg) => pg.env(app_name),
            DbInfo::Sqlite(db_path) => [
                ("GOLEM__DB__TYPE".to_string(), "Sqlite".to_string()),
                (
                    "GOLEM__DB__CONFIG__DATABASE".to_string(),
                    (db_path.join(app_name))
                        .to_str()
                        .expect("Invalid Sqlite database path")
                        .to_string(),
                ),
                (
                    "GOLEM__DB__CONFIG__MAX_CONNECTIONS".to_string(),
                    "10".to_string(),
                ),
            ]
            .into(),
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct PostgresInfo {
    #[arg(long = "postgres-host", default_value = "localhost")]
    pub host: String,
    #[arg(long = "postgres-port", default_value = "5432")]
    pub port: u16,
    #[arg(long = "postgres-host-port", default_value = "5432")]
    pub host_port: u16,
    #[arg(long = "postgres-username", default_value = "postgres")]
    pub username: String,
    #[arg(long = "postgres-password", default_value = "postgres")]
    pub password: String,
}

impl PostgresInfo {
    pub fn env(&self, app_name: &str) -> HashMap<String, String> {
        HashMap::from([
            ("DB_HOST".to_string(), self.host.clone()),
            ("DB_PORT".to_string(), self.port.to_string()),
            ("DB_NAME".to_string(), app_name.to_string()),
            ("DB_USERNAME".to_string(), self.username.clone()),
            ("DB_PASSWORD".to_string(), self.password.clone()),
            ("COMPONENT_REPOSITORY_TYPE".to_string(), "jdbc".to_string()),
            ("GOLEM__DB__TYPE".to_string(), "Postgres".to_string()),
            (
                "GOLEM__DB__CONFIG__MAX_CONNECTIONS".to_string(),
                "10".to_string(),
            ),
            ("GOLEM__DB__CONFIG__HOST".to_string(), self.host.clone()),
            ("GOLEM__DB__CONFIG__PORT".to_string(), self.port.to_string()),
            (
                "GOLEM__DB__CONFIG__DATABASE".to_string(),
                app_name.to_string(),
            ),
            (
                "GOLEM__DB__CONFIG__USERNAME".to_string(),
                self.username.clone(),
            ),
            (
                "GOLEM__DB__CONFIG__PASSWORD".to_string(),
                self.password.clone(),
            ),
        ])
    }
}

fn connection_string(host: &str, port: u16) -> String {
    format!("postgres://postgres:postgres@{host}:{port}/postgres?connect_timeout=3")
}

async fn check_if_running(host: &str, port: u16) -> Result<(), ::tokio_postgres::Error> {
    let (client, connection) =
        ::tokio_postgres::connect(&connection_string(host, port), ::tokio_postgres::NoTls).await?;

    let connection_fiber = tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });

    let r = client.simple_query("SELECT version();").await?;

    debug!("Test query returned with {r:?}");
    connection_fiber.abort();
    Ok(())
}

async fn wait_for_startup(host: &str, port: u16, timeout: Duration) {
    info!(
        "Waiting for Postgres start on host {host}:{port}, timeout: {}s",
        timeout.as_secs()
    );
    let start = Instant::now();
    loop {
        let running = check_if_running(host, port).await;

        match running {
            Ok(_) => break,
            Err(e) => {
                if start.elapsed() > timeout {
                    error!("Failed to verify that Postgres is running: {}", e);
                    std::panic!("Failed to verify that Postgres is running");
                }
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
}
