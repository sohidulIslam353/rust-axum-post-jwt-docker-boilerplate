use sea_orm::{Database, DatabaseConnection};
use std::env;

pub async fn connect_db() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in environment");
    Database::connect(&database_url).await
}

#[allow(dead_code)]
pub async fn close_db(db: DatabaseConnection) -> Result<(), sea_orm::DbErr> {
    db.close().await
}
