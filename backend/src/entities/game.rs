use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "games")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub lobby_id: Option<Uuid>,
    pub state: Json,
    pub created_at: DateTimeUtc,
    pub completed_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::lobby::Entity",
        from = "Column::LobbyId",
        to = "super::lobby::Column::Id"
    )]
    Lobby,
    #[sea_orm(has_many = "super::game_player::Entity")]
    Players,
    #[sea_orm(has_many = "super::game_round::Entity")]
    Rounds,
}

impl Related<super::lobby::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Lobby.def()
    }
}

impl Related<super::game_player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Players.def()
    }
}

impl Related<super::game_round::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Rounds.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
