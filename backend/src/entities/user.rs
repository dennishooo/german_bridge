use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    #[sea_orm(unique)]
    pub username: String,
    pub password_hash: String,
    pub created_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::lobby::Entity")]
    HostedLobbies,
    #[sea_orm(has_many = "super::lobby_player::Entity")]
    LobbyMemberships,
    #[sea_orm(has_many = "super::game_player::Entity")]
    GameParticipations,
}

impl Related<super::lobby::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::HostedLobbies.def()
    }
}

impl Related<super::lobby_player::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LobbyMemberships.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
