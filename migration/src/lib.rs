pub mod m20250909_145739_create_users_table;
pub use sea_orm_migration::prelude::*;
pub struct Migrator;
#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![Box::new(m20250909_145739_create_users_table::Migration)]
    }
}
