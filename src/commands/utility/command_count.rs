use std::sync::Arc;

use command_macros::{command, SlashCommand};
use time::OffsetDateTime;
use twilight_interactions::command::CreateCommand;

use crate::{
    core::commands::CommandOrigin, pagination::CommandCountPagination,
    util::interaction::InteractionCommand, Context, Result,
};

#[derive(CreateCommand, SlashCommand)]
#[command(name = "commands")]
#[flags(SKIP_DEFER)]
/// Display a list of popular commands
pub struct Commands;

pub async fn slash_commands(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    commands(ctx, (&mut command).into()).await
}

#[command]
#[desc("List of popular commands")]
#[group(Utility)]
#[flags(SKIP_DEFER)]
async fn prefix_commands(ctx: Arc<Context>, msg: &Message) -> Result<()> {
    commands(ctx, msg.into()).await
}

async fn commands(ctx: Arc<Context>, orig: CommandOrigin<'_>) -> Result<()> {
    let mut cmds: Vec<(String, u32)> = Vec::new();
    cmds.sort_unstable_by(|&(_, a), &(_, b)| b.cmp(&a));

    let booted_up = OffsetDateTime::now_utc();

    CommandCountPagination::builder(booted_up, cmds)
        .start(ctx, orig)
        .await
}