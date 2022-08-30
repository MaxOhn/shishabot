use std::fmt::Write;

use command_macros::EmbedData;
use twilight_model::channel::embed::EmbedField;

use crate::{commands::utility::GuildData, util::builder::AuthorBuilder};

use super::config::create_field;

#[derive(EmbedData)]
pub struct ServerConfigEmbed {
    author: AuthorBuilder,
    description: String,
    fields: Vec<EmbedField>,
    footer: &'static str,
    title: &'static str,
}

impl ServerConfigEmbed {
    pub fn new(guild: GuildData, config: GuildConfig, authorities: &[String]) -> Self {
        let mut author = AuthorBuilder::new(guild.name);

        if let Some(ref hash) = guild.icon {
            let url = format!(
                "https://cdn.discordapp.com/icons/{}/{hash}.{}",
                guild.id,
                if hash.is_animated() { "gif" } else { "webp" }
            );

            author = author.icon_url(url);
        }

        let title = "Current server configuration:";

        let mut description = String::with_capacity(256);

        description.push_str("```\nAuthorities: ");

        let mut authorities = authorities.iter();

        if let Some(auth) = authorities.next() {
            let _ = write!(description, "@{auth}");

            for auth in authorities {
                let _ = write!(description, ", @{auth}");
            }
        } else {
            description.push_str("None");
        }

        description.push_str("\nPrefixes: ");
        let mut prefixes = config.prefixes.iter();

        if let Some(prefix) = prefixes.next() {
            let _ = write!(description, "`{prefix}`");

            for prefix in prefixes {
                let _ = write!(description, ", `{prefix}`");
            }
        }

        let track_limit = config.track_limit();
        let _ = writeln!(description, "\nDefault track limit: {track_limit}\n```");

        let fields = vec![
            create_field(
                "Song commands",
                config.with_lyrics(),
                &[(true, "enabled"), (false, "disabled")],
            ),
            create_field(
                "Retries*",
                config.show_retries(),
                &[(true, "show"), (false, "hide")],
            ),
        ];

        Self {
            author,
            description,
            fields,
            footer: "*: Only applies if not set in the member's user config",
            title,
        }
    }
}
