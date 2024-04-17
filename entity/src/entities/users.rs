//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique)]
    pub user_name: String,
    #[sea_orm(unique)]
    pub public_key: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::nativetokenbalance::Entity")]
    Nativetokenbalance,
    #[sea_orm(has_many = "super::usertokens::Entity")]
    Usertokens,
}

impl Related<super::nativetokenbalance::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Nativetokenbalance.def()
    }
}

impl Related<super::usertokens::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Usertokens.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
