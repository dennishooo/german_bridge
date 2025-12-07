pub mod migration;
pub use sea_orm_migration::prelude::*;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(migration::m20241207_000001_create_tables::Migration),
            Box::new(migration::m20251207_025543_add_current_round::Migration),
        ]
    }
}
