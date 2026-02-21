pub use sea_orm_migration::prelude::*;

mod entities;

mod m20251218_164925_create_entries;
mod m20251219_084835_create_topics_users;
mod m20251219_091509_create_tasks;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20251218_164925_create_entries::Migration),
            Box::new(m20251219_084835_create_topics_users::Migration),
            Box::new(m20251219_091509_create_tasks::Migration),
        ]
    }
}
