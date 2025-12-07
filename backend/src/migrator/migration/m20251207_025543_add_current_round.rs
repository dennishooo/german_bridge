use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop old columns
        manager
            .alter_table(
                Table::alter()
                    .table(GameRounds::Table)
                    .drop_column(GameRounds::Bids)
                    .drop_column(GameRounds::TricksWon)
                    .drop_column(GameRounds::Scores)
                    .to_owned(),
            )
            .await?;

        // Add new player_results column
        manager
            .alter_table(
                Table::alter()
                    .table(GameRounds::Table)
                    .add_column(
                        ColumnDef::new(GameRounds::PlayerResults)
                            .json()
                            .not_null()
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Drop player_results column
        manager
            .alter_table(
                Table::alter()
                    .table(GameRounds::Table)
                    .drop_column(GameRounds::PlayerResults)
                    .to_owned(),
            )
            .await?;

        // Re-add old columns
        manager
            .alter_table(
                Table::alter()
                    .table(GameRounds::Table)
                    .add_column(ColumnDef::new(GameRounds::Bids).json().not_null())
                    .add_column(ColumnDef::new(GameRounds::TricksWon).json().not_null())
                    .add_column(ColumnDef::new(GameRounds::Scores).json().not_null())
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum GameRounds {
    Table,
    Bids,
    TricksWon,
    Scores,
    PlayerResults,
}
