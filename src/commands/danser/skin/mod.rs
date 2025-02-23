use std::sync::Arc;

use command_macros::SlashCommand;
use eyre::Result;
use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::channel::Attachment;

use crate::{
    core::Context,
    util::{interaction::InteractionCommand, InteractionCommandExt},
};

use self::{add::*, remove::*};

mod add;
mod remove;

#[derive(CommandModel, CreateCommand, SlashCommand)]
#[command(name = "skin")]
#[flags(ONLY_OWNER, SKIP_DEFER)]
/// Skinlist configuration
pub enum Skin {
    #[command(name = "add")]
    Add(SkinAdd),
    #[command(name = "remove")]
    Remove(SkinRemove),
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "add")]
/// Add a skin to the skinlist
pub struct SkinAdd {
    /// Skin that you want to add
    skin: Attachment,
}

#[derive(CommandModel, CreateCommand)]
#[command(name = "remove")]
/// Remove a skin to the skinlist
pub struct SkinRemove {
    /// Index of the skin that you want to remove
    #[command(min_value = 0, max_value = 65_535)]
    index: usize,
}

pub async fn slash_skin(ctx: Arc<Context>, mut command: InteractionCommand) -> Result<()> {
    match Skin::from_interaction(command.input_data())? {
        Skin::Add(args) => add(ctx, command, args).await,
        Skin::Remove(args) => remove(ctx, command, args).await,
    }
}
