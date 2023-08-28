mod service_reserve_tests;

#[cfg(test)]
mod test_utils {
    use abi::Config;
    use sqlx::{Connection, Executor, PgConnection};
    use std::sync::Arc;
    use uuid::Uuid;

    pub(crate) struct TestConfig {
        pub config: Arc<Config>,
    }

    impl TestConfig {
        // create a new config and initialize
        pub async fn new() -> Self {
            let mut config = Config::load("../service/fixtures/config.yml").unwrap();
            let uuid = Uuid::new_v4();
            let dbname = format!("test_service_{}", uuid);
            config.db.dbname = dbname;

            Self {
                config: Arc::new(config),
            }
        }

        // make test database same as real database
        pub async fn initialize(&self) {
            let server_url = self.config.db.server_url();
            let url = self.config.db.database_url();

            // use server url to make connection to database server
            let mut conn = PgConnection::connect(&server_url).await.unwrap();
            conn.execute(format!(r#"CREATE DATABASE "{}""#, self.config.db.dbname).as_str())
                .await
                .unwrap();

            // to make sure disconnect to database server
            drop(conn);

            // now connect to just created test database for migration
            let mut conn = PgConnection::connect(&url).await.unwrap();
            sqlx::migrate!("../migrations")
                .run(&mut conn)
                .await
                .unwrap();
        }

        // after the test, need to terminate all connections and Drop the database
        pub async fn cleanup(&self) {
            let server_url = self.config.db.server_url();
            let dbname = self.config.db.dbname.clone();

            let mut conn = sqlx::PgConnection::connect(&server_url).await.unwrap();

            // terminate existing connections MAGIC MDFK
            sqlx::query(&format!(r#"SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND datname = '{}'"#, dbname))
            .execute(&mut conn)
            .await
            .expect("Terminate all other connections");

            conn.execute(format!(r#"DROP DATABASE "{}""#, dbname).as_str())
                .await
                .expect("Error while querying the drop database");
        }
    }
}
