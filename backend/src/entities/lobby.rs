use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lobbies")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub host_id: Uuid,
    pub max_players: i32,
    pub settings: Json,
    pub created_at: DateTimeUtc,
    pub closed_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::HostId",
        to = "super::user::Column::Id"
    )]
    Host,
    #[sea_orm(has_many = "super::lobby_player::Entity")]
    Players,
    #[sea_orm(has_many = "super::game::Entity")]
    Games,
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Host.def()
    }
}

impl Related<super::lobby_player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Players.def()
    }
}

impl Related<super::game::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Games.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
