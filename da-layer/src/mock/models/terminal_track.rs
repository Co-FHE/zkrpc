//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.15

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "terminal_track")]
pub struct Model {
    pub block_number: i32,
    pub satellite_mac: String,
    pub terminal_mac: String,
    #[sea_orm(column_type = "Float")]
    pub signal_strength: f32,
    pub net_bandwidth: i32,
    pub net_traffic: i32,
    pub connect_time: i64,
    pub disconnect_time: Option<i64>,
    pub net_latency: i32,
    pub droped_ip_packets: Option<String>,
    pub terminal_address: String,
    pub satellite_validator_address: String,
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(column_type = "Float", nullable)]
    pub latitude: Option<f32>,
    #[sea_orm(column_type = "Float", nullable)]
    pub longitude: Option<f32>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::terminal::Entity",
        from = "Column::TerminalAddress",
        to = "super::terminal::Column::Address",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Terminal,
}

impl Related<super::terminal::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Terminal.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
