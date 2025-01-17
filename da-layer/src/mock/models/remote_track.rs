//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "remote_track")]
pub struct Model {
    pub remote_mac: String,
    pub block_number: i32,
    #[sea_orm(column_type = "Float")]
    pub y: f32,
    #[sea_orm(column_type = "Float")]
    pub x: f32,
    #[sea_orm(column_type = "Float")]
    pub height: f32,
    #[sea_orm(column_type = "Float")]
    pub speed: f32,
    pub bandwidth_ground: i32,
    pub bandwidth_space: i32,
    pub validator_address: String,
    #[sea_orm(primary_key)]
    pub id: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
