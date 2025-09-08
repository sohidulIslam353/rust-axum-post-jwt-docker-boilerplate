use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .if_not_exists()
                    // Primary key with auto increment
                    .col(
                        ColumnDef::new(Category::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    // Category name
                    .col(ColumnDef::new(Category::Name).string().not_null())
                    // Slug (unique)
                    .col(
                        ColumnDef::new(Category::Slug)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    // Status (Active/Inactive)
                    .col(
                        ColumnDef::new(Category::Status)
                            .string()
                            .not_null()
                            .default("Active"),
                    )
                    // Optional: created_at & updated_at timestamps
                    .col(
                        ColumnDef::new(Category::CreatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(Category::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Category::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Category {
    Table,
    Id,
    Name,
    Slug,
    Status,
    CreatedAt,
    UpdatedAt,
}
