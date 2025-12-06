use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create users table
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Users::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Users::Username).string_len(50).not_null().unique_key())
                    .col(ColumnDef::new(Users::PasswordHash).text().not_null())
                    .col(ColumnDef::new(Users::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .to_owned(),
            )
            .await?;

        // Create lobbies table
        manager
            .create_table(
                Table::create()
                    .table(Lobbies::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Lobbies::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Lobbies::HostId).uuid().not_null())
                    .col(ColumnDef::new(Lobbies::MaxPlayers).integer().not_null().default(4))
                    .col(ColumnDef::new(Lobbies::Settings).json_binary().not_null())
                    .col(ColumnDef::new(Lobbies::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Lobbies::ClosedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Lobbies::Table, Lobbies::HostId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await?;

        // Create lobby_players table
        manager
            .create_table(
                Table::create()
                    .table(LobbyPlayers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(LobbyPlayers::LobbyId).uuid().not_null())
                    .col(ColumnDef::new(LobbyPlayers::PlayerId).uuid().not_null())
                    .col(ColumnDef::new(LobbyPlayers::JoinedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .primary_key(Index::create().col(LobbyPlayers::LobbyId).col(LobbyPlayers::PlayerId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(LobbyPlayers::Table, LobbyPlayers::LobbyId)
                            .to(Lobbies::Table, Lobbies::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(LobbyPlayers::Table, LobbyPlayers::PlayerId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await?;

        // Create games table
        manager
            .create_table(
                Table::create()
                    .table(Games::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(Games::Id).uuid().not_null().primary_key())
                    .col(ColumnDef::new(Games::LobbyId).uuid().null())
                    .col(ColumnDef::new(Games::State).json_binary().not_null())
                    .col(ColumnDef::new(Games::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                    .col(ColumnDef::new(Games::CompletedAt).timestamp_with_time_zone().null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(Games::Table, Games::LobbyId)
                            .to(Lobbies::Table, Lobbies::Id)
                    )
                    .to_owned(),
            )
            .await?;

        // Create game_players table
        manager
            .create_table(
                Table::create()
                    .table(GamePlayers::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GamePlayers::GameId).uuid().not_null())
                    .col(ColumnDef::new(GamePlayers::PlayerId).uuid().not_null())
                    .col(ColumnDef::new(GamePlayers::FinalScore).integer().null())
                    .primary_key(Index::create().col(GamePlayers::GameId).col(GamePlayers::PlayerId))
                    .foreign_key(
                        ForeignKey::create()
                            .from(GamePlayers::Table, GamePlayers::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(GamePlayers::Table, GamePlayers::PlayerId)
                            .to(Users::Table, Users::Id)
                    )
                    .to_owned(),
            )
            .await?;

        // Create game_rounds table
        manager
            .create_table(
                Table::create()
                    .table(GameRounds::Table)
                    .if_not_exists()
                    .col(ColumnDef::new(GameRounds::Id).integer().not_null().auto_increment().primary_key())
                    .col(ColumnDef::new(GameRounds::GameId).uuid().not_null())
                    .col(ColumnDef::new(GameRounds::RoundNumber).integer().not_null())
                    .col(ColumnDef::new(GameRounds::Bids).json_binary().not_null())
                    .col(ColumnDef::new(GameRounds::TricksWon).json_binary().not_null())
                    .col(ColumnDef::new(GameRounds::Scores).json_binary().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(GameRounds::Table, GameRounds::GameId)
                            .to(Games::Table, Games::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(GameRounds::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(GamePlayers::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Games::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(LobbyPlayers::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Lobbies::Table).to_owned()).await?;
        manager.drop_table(Table::drop().table(Users::Table).to_owned()).await?;
        Ok(())
    }
}

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Username,
    PasswordHash,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Lobbies {
    Table,
    Id,
    HostId,
    MaxPlayers,
    Settings,
    CreatedAt,
    ClosedAt,
}

#[derive(DeriveIden)]
enum LobbyPlayers {
    Table,
    LobbyId,
    PlayerId,
    JoinedAt,
}

#[derive(DeriveIden)]
enum Games {
    Table,
    Id,
    LobbyId,
    State,
    CreatedAt,
    CompletedAt,
}

#[derive(DeriveIden)]
enum GamePlayers {
    Table,
    GameId,
    PlayerId,
    FinalScore,
}

#[derive(DeriveIden)]
enum GameRounds {
    Table,
    Id,
    GameId,
    RoundNumber,
    Bids,
    TricksWon,
    Scores,
}
