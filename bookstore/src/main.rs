use bookstore::{appstate::AppState, create_router, run_migrations};
use color_eyre::eyre::Context;
use sqlx::{Postgres, pool::PoolOptions};
use std::{env, time::Duration};
use tracing::info;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install().unwrap();
    tracing_subscriber::fmt::init();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PoolOptions::<Postgres>::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await
        .wrap_err("Could not connect to database")?;

    {
        let mut conn = pool.acquire().await?;
        run_migrations(&mut conn).await?;
    }

    let app = create_router(AppState::new(pool));

    let bindto = std::env::var("BIND_TO").unwrap_or("127.0.0.1:3000".to_string());
    println!("Starting web server on {bindto}");
    let addr = tokio::net::TcpListener::bind(bindto).await?;

    axum::serve(addr, app).await?;

    Ok(())
}
