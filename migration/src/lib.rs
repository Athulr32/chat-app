pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20240312_144152_user_table_constraint;
mod m20240316_053714_remove_constraint;
mod m20240316_094538_remove_constraints;
mod m20240413_131231_add_native_token_balance_table;
mod m20240417_141245_add_contraint_to_native_token_table;
mod m20240422_190442_user_tokens;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20240312_144152_user_table_constraint::Migration),
            Box::new(m20240316_053714_remove_constraint::Migration),
            Box::new(m20240316_094538_remove_constraints::Migration),
            Box::new(m20240413_131231_add_native_token_balance_table::Migration),
            Box::new(m20240417_141245_add_contraint_to_native_token_table::Migration),
            Box::new(m20240422_190442_user_tokens::Migration),
        ]
    }
}
