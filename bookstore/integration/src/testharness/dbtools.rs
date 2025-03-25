use std::time::Duration;

use color_eyre::{Help, eyre::Context};
use sqlx::{PgPool, Pool, Postgres, migrate, postgres::PgPoolOptions, query};
use tracing::{debug, log::warn};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AdminDatabaseConnection {
    pool: Pool<Postgres>,
}

fn connect_with_dbname(database: &str) -> String {
    let urlstring =
        std::env::var("TEST_DATABASE_URL").expect("Environment variable TEST_DATABASE_URL missing");
    let url = format!("{urlstring}/{database}");
    url
}

impl AdminDatabaseConnection {
    pub async fn drop_application_pool(&self, app_pool: Pool<Postgres>) {
        let dbname = app_pool
            .connect_options()
            .get_database()
            .unwrap()
            .to_string();
        app_pool.close().await;
        // Have to drop app_pool here, otherwise the connection stays open until it goes out of
        // scope, which causes the DROP DATABASE to fail, since ther are existing connections to
        // the database
        drop(app_pool);
        debug!("Dropping database");
        let mut executor = self.pool.acquire().await.unwrap();
        if let Err(e) = query(&format!("DROP DATABASE {dbname}"))
            .execute(&mut *executor)
            .await
        {
            warn!(
                "Could not delete database '{}', it's lingering now: {}",
                dbname, e
            );
        }
    }

    pub async fn create_application_pool(&self, test_name: &str) -> Result<PgPool, sqlx::Error> {
        let name = format!("{test_name}-{}", Uuid::new_v4().hyphenated()).replace('-', "_");
        debug!(db = name, "Creating database");
        let mut executor = self.pool.acquire().await.unwrap();
        query(&format!("CREATE DATABASE {name}"))
            .execute(&mut *executor)
            .await
            .expect("Could not create appication pool");

        debug!(db = name, "Created");

        let pool = PgPoolOptions::new()
            .min_connections(1)
            .max_connections(4)
            .connect(&connect_with_dbname(&name))
            .await?;

        debug!(db = name, "Running migrations");

        migrate!("./migrations").run(&pool).await?;

        Ok(pool)
    }
}

pub async fn create_administrative_database() -> AdminDatabaseConnection {
    let connect_options = connect_with_dbname("postgres");
    let pool = PgPoolOptions::new()
        .min_connections(1)
        .max_connections(20)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&connect_options)
        .await
        .wrap_err("Could not set up the administrative connection")
        .suggestion(format!(
            "Is the test database running? It's expected at {connect_options}"
        ))
        .suggestion(r#"You can run one with: "docker run --rm --name test-postgres -e POSTGRES_PASSWORD=root -p 5433:5432 --mount type=tmpfs,destination=/var/lib/postgresql/data -d postgres""#)
        .unwrap();

    debug!("Administrative connection is up");

    AdminDatabaseConnection { pool }
}

pub async fn get_admin_pool() -> &'static AdminDatabaseConnection {
    static ADMIN_POOL: tokio::sync::OnceCell<AdminDatabaseConnection> =
        tokio::sync::OnceCell::const_new();
    ADMIN_POOL.get_or_init(create_administrative_database).await
}
