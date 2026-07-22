use sea_orm::{Database, DatabaseConnection};

pub async fn init() -> anyhow::Result<DatabaseConnection> {
    let db_url = dotenv::var("DATABASE_URL")?;
    let db = Database::connect(db_url).await?;
    Ok(db)
}