use std::sync::Arc;

use eyre::Result;
use twilight_model::{channel::Message, guild::Permissions};

use crate::{
    core::{
        buckets::BucketName,
        commands::{
            checks::{check_authority, check_ratelimit},
            prefix::{Args, PrefixCommand, Stream},
        },
        Context,
    },
    util::ChannelExt,
    DEFAULT_PREFIX,
};

use self::parse::*;

use super::{log_command, ProcessResult};

mod parse;

pub async fn handle_message(ctx: Arc<Context>, msg: Message) {
    // Ignore bots and webhooks
    if msg.author.bot || msg.webhook_id.is_some() {
        return;
    }

    // Check msg content for a prefix
    let mut stream = Stream::new(&msg.content);
    stream.take_while_char(char::is_whitespace);

    let prefix = match msg.guild_id {
        Some(guild_id) => ctx.guild_prefixes_find(guild_id, &stream),
        None => stream
            .starts_with(DEFAULT_PREFIX)
            .then(|| DEFAULT_PREFIX.into()),
    };

    if let Some(prefix) = prefix {
        stream.increment(prefix.len());
    } else if msg.guild_id.is_some() {
        return;
    }

    // Parse msg content for commands
    let (cmd, num) = match parse_invoke(&mut stream) {
        Invoke::Command { cmd, num } => (cmd, num),
        Invoke::None => return,
    };

    let name = cmd.name();
    log_command(&ctx, &msg, name);

    match process_command(ctx, cmd, &msg, stream, num).await {
        Ok(ProcessResult::Success) => info!("Processed command `{name}`"),
        Ok(result) => info!("Command `{name}` was not processed: {result:?}"),
        Err(err) => {
            error!("failed to process prefix command `{name}`: {err:?}");
        }
    }
}

async fn process_command(
    ctx: Arc<Context>,
    cmd: &PrefixCommand,
    msg: &Message,
    stream: Stream<'_>,
    num: Option<u64>,
) -> Result<ProcessResult> {
    // Only in guilds?
    if (cmd.flags.authority() || cmd.flags.only_guilds()) && msg.guild_id.is_none() {
        let content = "That command is only available in servers";
        msg.error(&ctx, content).await?;

        return Ok(ProcessResult::NoDM);
    }

    // Only for owner?
    // * Not necessary since there are no owner-only prefix commands

    let channel = msg.channel_id;

    // Does bot have sufficient permissions to send response in a guild?
    if let Some(guild) = msg.guild_id {
        let user = ctx.cache.current_user(|user| user.id)?;

        let permissions = ctx
            .cache
            .get_channel_permissions(user, channel, Some(guild));

        if !permissions.contains(Permissions::SEND_MESSAGES) {
            return Ok(ProcessResult::NoSendPermission);
        }
    }

    // Ratelimited?
    let ratelimit = ctx
        .buckets
        .get(BucketName::All)
        .lock()
        .unwrap()
        .take(msg.author.id.get());

    if ratelimit > 0 {
        trace!(
            "Ratelimiting user {} for {ratelimit} seconds",
            msg.author.id,
        );

        return Ok(ProcessResult::Ratelimited(BucketName::All));
    }

    if let Some(bucket) = cmd.bucket {
        if let Some(cooldown) = check_ratelimit(&ctx, msg.author.id, bucket).await {
            trace!(
                "Ratelimiting user {} on bucket `{bucket:?}` for {cooldown} seconds",
                msg.author.id,
            );

            let content = format!("Command on cooldown, try again in {cooldown} seconds");
            msg.error(&ctx, content).await?;

            return Ok(ProcessResult::Ratelimited(bucket));
        }
    }

    // Only for authorities?
    if cmd.flags.authority() {
        match check_authority(&ctx, msg.author.id, msg.channel_id, msg.guild_id).await {
            None => {}
            Some(content) => {
                let _ = msg.error(&ctx, content).await;

                return Ok(ProcessResult::NoAuthority);
            }
        }
    }

    // Prepare lightweight arguments
    let args = Args::new(&msg.content, stream, num);

    // Broadcast typing event
    if cmd.flags.defer() {
        let _ = ctx.http.create_typing_trigger(channel).exec().await;
    }

    // Call command function
    (cmd.exec)(ctx, msg, args).await?;

    Ok(ProcessResult::Success)
}
