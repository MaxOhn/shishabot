use std::{env, path::PathBuf};

use eyre::{Context, ContextCompat, Result};
use once_cell::sync::OnceCell;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, UserMarker},
    Id,
};

static CONFIG: OnceCell<BotConfig> = OnceCell::new();

#[derive(Debug)]
pub struct BotConfig {
    pub tokens: Tokens,
    pub paths: Paths,
    pub owners: Vec<Id<UserMarker>>,
    pub dev_guild: Id<GuildMarker>,
}

#[derive(Debug)]
pub struct Paths {
    pub folders: PathBuf,
    pub maps: PathBuf,
    pub server_settings: PathBuf,
}

#[derive(Debug)]
pub struct Tokens {
    pub discord: String,
    pub osu_client_id: u64,
    pub osu_client_secret: String,
}

impl BotConfig {
    pub fn get() -> &'static Self {
        CONFIG
            .get()
            .expect("`BotConfig::init` must be called first")
    }

    pub fn init() -> Result<()> {
        let config = BotConfig {
            tokens: Tokens {
                discord: env_var("DISCORD_TOKEN")?,
                osu_client_id: env_var("OSU_CLIENT_ID")?,
                osu_client_secret: env_var("OSU_CLIENT_SECRET")?,
            },
            paths: Paths {
                folders: env_var("FOLDERS_PATH")?,
                maps: env_var("MAP_PATH")?,
                server_settings: env_var("SERVER_SETTINGS_PATH")?,
            },
            owners: env_var("OWNERS_USER_ID")?,
            dev_guild: env_var("DEV_GUILD_ID")?,
        };

        if CONFIG.set(config).is_err() {
            error!("CONFIG was already set");
        }

        Ok(())
    }
}

trait EnvKind: Sized {
    const EXPECTED: &'static str;

    fn from_str(s: &str) -> Option<Self>;
}

macro_rules! env_kind {
    ($($ty:ty: $arg:ident => $impl:block,)*) => {
        $(
            impl EnvKind for $ty {
                const EXPECTED: &'static str = stringify!($ty);

                fn from_str($arg: &str) -> Option<Self> {
                    $impl
                }
            }
        )*
    };
}

env_kind! {
    u16: s => { s.parse().ok() },
    u64: s => { s.parse().ok() },
    PathBuf: s => { s.parse().ok() },
    String: s => { Some(s.to_owned()) },
    Id<UserMarker>: s => { s.parse().ok().map(Id::new) },
    Id<GuildMarker>: s => { s.parse().ok().map(Id::new) },
    Id<ChannelMarker>: s => { s.parse().ok().map(Id::new) },
    Vec<Id<UserMarker>>: s => {
        if !(s.starts_with('[') && s.ends_with(']')) {
            return None
        }

        s[1..s.len() - 1]
            .split(',')
            .map(str::trim)
            .map(str::parse)
            .collect::<Result<Vec<_>, _>>()
            .ok()
    },
}

fn env_var<T: EnvKind>(name: &'static str) -> Result<T> {
    let value = env::var(name).with_context(|| format!("missing env variable `{name}`"))?;

    T::from_str(&value).with_context(|| {
        format!(
            "failed to parse env variable `{name}={value}`; expected {expected}",
            expected = T::EXPECTED
        )
    })
}
