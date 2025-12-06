use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "lobby_players")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub lobby_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub player_id: Uuid,
    pub joined_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::lobby::Entity",
        from = "Column::LobbyId",
        to = "super::lobby::Column::Id"
    )]
    Lobby,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::PlayerId",
        to = "super::user::Column::Id"
    )]
    Player,
}

impl Related<super::lobby::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lobby.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Player.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
