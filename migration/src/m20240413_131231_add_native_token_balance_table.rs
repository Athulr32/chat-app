use async_std::os::unix::net::UnixListener;
use sea_orm::{sea_query::*, EntityName, Schema};
use sea_orm_migration::prelude::*;
#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        let db = manager.get_connection();
        db.execute_unprepared(
            "CREATE TABLE NativeTokenBalance (
                Id SERIAL PRIMARY KEY,
                public_key VARCHAR(255),
                token_name VARCHAR(255),
                token_balance BIGINT,
                FOREIGN KEY (public_key) REFERENCES Users(public_key)
            );
            ;",
        )
        .await?;

        Ok(())

        // manager
        //     .create_table(
        //         Table::create()
        //             .table(Post::NativeTokenBalance)
        //             .if_not_exists()
        //             .col(ColumnDef::new(Post::Balance).big_unsigned().not_null())
        //             .col(ColumnDef::new(Post::Token).string().not_null())
        //             .col(ColumnDef::new(Post::UserAddress).primary_key())
        //             .foreign_key(ForeignKeyCreateStatement::new().name("Native_token_constraint")
        //                 .from_tbl(  )
        //                 .from_col("public_key")
        //                 .to_tbl(Post)
        //                 .to_col(Post::UserAddress)
        //             )
        //             .to_owned(),
        //     )
        //     .await?;

        //     Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        todo!();
    }
}

#[derive(DeriveIden)]
enum Post {
    NativeTokenBalance,
    UserAddress,
    Token,
    Balance,
}
