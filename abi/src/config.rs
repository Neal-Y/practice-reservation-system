use crate::Error;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Config {
    pub db: DbConfig,
    pub server: SeverConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub dbname: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

fn default_max_connections() -> u32 {
    5
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeverConfig {
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load(filename: &str) -> Result<Self, Error> {
        let config = fs::read_to_string(filename).map_err(|_| Error::FailedToParse)?;
        serde_yaml::from_str(&config).map_err(|_| Error::FailedToRead)
    }
}

impl DbConfig {
    pub fn server_url(&self) -> String {
        if self.password.is_empty() {
            format!("postgres://{}@{}:{}", self.user, self.host, self.port)
        } else {
            format!(
                "postgres://{}:{}@{}:{}",
                self.user, self.password, self.host, self.port
            )
        }
    }

    pub fn database_url(&self) -> String {
        format!("{}/{}", self.server_url(), self.dbname)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn config_should_be_loaded() {
        let config = Config::load("../service/fixtures/config.yml").unwrap();
        assert_eq!(
            config,
            Config {
                db: DbConfig {
                    host: "localhost".to_string(),
                    port: 5432,
                    user: "postgres".to_string(),
                    password: "".to_string(),
                    dbname: "reservation".to_string(),
                    max_connections: 5
                },
                server: SeverConfig {
                    host: "localhost".to_string(),
                    port: 8080
                }
            }
        )
    }
}
