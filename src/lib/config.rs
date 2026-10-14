use figment::{
    providers::{Env, Format, Toml},
    Error, Figment,
};
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct Config {
    #[serde(alias = "DISCORD_TOKEN")]
    pub discord_token: String,
    #[serde(alias = "SPOTIFY_USERNAME")]
    #[serde(default)]
    pub spotify_username: String,
    #[serde(alias = "SPOTIFY_PASSWORD")]
    #[serde(default)]
    pub spotify_password: String,
    #[serde(alias = "DISCORD_USER_ID")]
    pub discord_user_id: u64,
    #[serde(alias = "SPOTIFY_BOT_AUTOPLAY")]
    #[serde(default)]
    pub spotify_bot_autoplay: bool,
    #[serde(alias = "SPOTIFY_DEVICE_NAME")]
    #[serde(default = "default_spotify_device_name")]
    pub spotify_device_name: String,
    #[serde(alias = "CACHE_DIR")]
    #[serde(default = "default_cache_dir")]
    pub cache_dir: String,
}

fn default_spotify_device_name() -> String {
    "Aoede".to_string()
}

fn default_cache_dir() -> String {
    "aoede-cache".to_string()
}

impl Config {
    pub fn new() -> Result<Self, Box<Error>> {
        let config: Config = Figment::new()
            .merge(Toml::file("config.toml"))
            .merge(Env::raw())
            .extract()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_credentials_do_not_require_username_or_password() {
        let config: Config = Figment::from(Toml::string(
            r#"
            discord_token = "example-bot-token"
            discord_user_id = 123456789
            cache_dir = "/data"
            "#,
        ))
        .extract()
        .unwrap();

        assert!(config.spotify_username.is_empty());
        assert!(config.spotify_password.is_empty());
        assert_eq!(config.cache_dir, "/data");
        assert_eq!(config.discord_user_id, 123456789);
        assert!(!config.spotify_bot_autoplay);
        assert_eq!(config.spotify_device_name, "Aoede");
    }

    #[test]
    fn discord_user_id_must_be_numeric() {
        let config = Figment::from(Toml::string(
            r#"
            discord_token = "example-bot-token"
            discord_user_id = "example-user"
            "#,
        ))
        .extract::<Config>();

        assert!(config.is_err());
    }
}
