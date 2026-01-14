use std::env;

use tokio::sync::OnceCell;

#[derive(Debug)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug)]
struct DatabaseConfig {
    url: String,
}

#[derive(Debug)]
struct AppConfig {
    twitch_oauth: OAuthConfig,
    streamlabs_oauth: OAuthConfig,
    client_version: String,
}

#[derive(Debug)]
pub struct OAuthConfig {
    redirect: String,
    scopes: String,
    client: String,
    secret: String,
}

impl OAuthConfig {
    pub fn redirect(&self) -> &str {
        &self.redirect
    }

    pub fn scopes(&self) -> &str {
        &self.scopes
    }

    pub fn client(&self) -> &str {
        &self.client
    }

    pub fn secret(&self) -> &str {
        &self.secret
    }
}

#[derive(Debug)]
pub struct Config {
    server: ServerConfig,
    db: DatabaseConfig,
    app: AppConfig,
}

impl Config {
    pub fn db_url(&self) -> &str {
        &self.db.url
    }

    pub fn server_host(&self) -> &str {
        &self.server.host
    }

    pub fn server_port(&self) -> u16 {
        self.server.port
    }

    pub fn twitch_oauth(&self) -> &OAuthConfig {
        &self.app.twitch_oauth
    }

    pub fn streamlabs_oauth(&self) -> &OAuthConfig {
        &self.app.streamlabs_oauth
    }

    pub fn client_version(&self) -> &str {
        &self.app.client_version
    }
}

pub static CONFIG: OnceCell<Config> = OnceCell::const_new();

async fn init_config() -> Config {
    dotenv::dotenv().ok();

    let server_config = ServerConfig {
        host: env::var("HOST").unwrap_or_else(|_| String::from("127.0.0.1")),
        port: env::var("PORT")
            .unwrap_or_else(|_| String::from("3000"))
            .parse::<u16>()
            .unwrap(),
    };

    let database_config = DatabaseConfig {
        url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
    };

    let app_config = AppConfig {
        twitch_oauth: OAuthConfig {
            redirect: env::var("TWITCH_REDIRECT").expect("TWITCH_REDIRECT must be set"),
            scopes: env::var("TWITCH_SCOPES").expect("TWITCH_SCOPES must be set"),
            client: env::var("TWITCH_CLIENT").expect("TWITCH_CLIENT must be set"),
            secret: env::var("TWITCH_SECRET").expect("TWITCH_SECRET must be set"),
        },
        streamlabs_oauth: OAuthConfig {
            redirect: env::var("STREAMLABS_REDIRECT").expect("STREAMLABS_REDIRECT must be set"),
            scopes: env::var("STREAMLABS_SCOPES").expect("STREAMLABS_SCOPES must be set"),
            client: env::var("STREAMLABS_CLIENT").expect("STREAMLABS_CLIENT must be set"),
            secret: env::var("STREAMLABS_SECRET").expect("STREAMLABS_SECRET must be set"),
        },
        client_version: env::var("CLIENT_VERSION").expect("CLIENT_VERSION must be set"),
    };

    Config {
        server: server_config,
        db: database_config,
        app: app_config,
    }
}

pub async fn config() -> &'static Config {
    CONFIG.get_or_init(init_config).await
}
