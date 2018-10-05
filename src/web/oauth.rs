use oauth2::basic::BasicClient;
use oauth2::prelude::*;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, Scope, TokenUrl};

use rocket::http::RawStr;
use rocket::request::FromFormValue;

use std::env;
use url::Url;

lazy_static! {
    static ref DROPBOX_CONFIG: Oauth2Config = {
        info!("Initializing Dropbox oauth config");
        Oauth2Config::dropbox()
    };
    static ref YOUTUBE_CONFIG: Oauth2Config = {
        info!("Initializing Youtube oauth config");
        Oauth2Config::youtube()
    };
}

lazy_static! {
    static ref BASE_URL: Url = Url::parse(
        &env::var("ARCHIVER_BASE_URL")
            .expect("Missing the ARCHIVER_BASE_URL environment variable."),
    ).expect("Invalid ARCHIVER_BASE_URL");
}

#[derive(Clone)]
pub struct Oauth2Config {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub auth_url: AuthUrl,
    pub token_url: TokenUrl,
    pub scopes: &'static [&'static str],
    pub redirect_url: RedirectUrl,
}

impl Oauth2Config {
    /// Creates a Oauth2Config configured for Dropbox, panicing on many types of failure, since
    /// they are all unrecoverable.
    pub fn dropbox() -> Oauth2Config {
        let client_id = ClientId::new(
            env::var("ARCHIVER_DROPBOX_APP_KEY")
                .expect("Missing the ARCHIVER_DROPBOX_APP_KEY environment variable."),
        );
        let client_secret = ClientSecret::new(
            env::var("ARCHIVER_DROPBOX_APP_SECRET")
                .expect("Missing the ARCHIVER_DROPBOX_APP_SECRET environment variable."),
        );
        let auth_url = AuthUrl::new(
            Url::parse("https://www.dropbox.com/oauth2/authorize")
                .expect("Invalid authorization endpoint URL"),
        );
        let token_url = TokenUrl::new(
            Url::parse("https://www.dropbox.com/oauth2/token").expect("Invalid token endpoint URL"),
        );
        let redirect_url = RedirectUrl::new(
            BASE_URL
                .join("/integration/finish?provider=dropbox")
                .expect("Invalid redirect URL"),
        );
        let scopes = &[];
        Oauth2Config {
            client_id,
            client_secret,
            auth_url,
            token_url,
            scopes,
            redirect_url,
        }
    }
    pub fn youtube() -> Oauth2Config {
        let client_id = ClientId::new(
            env::var("ARCHIVER_YOUTUBE_APP_KEY")
                .expect("Missing the ARCHIVER_YOUTUBE_APP_KEY environment variable."),
        );
        let client_secret = ClientSecret::new(
            env::var("ARCHIVER_YOUTUBE_APP_SECRET")
                .expect("Missing the ARCHIVER_YOUTUBE_APP_SECRET environment variable."),
        );
        let auth_url = AuthUrl::new(
            Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
                .expect("Invalid authorization endpoint URL"),
        );
        let token_url = TokenUrl::new(
            Url::parse("https://www.googleapis.com/oauth2/v4/token")
                .expect("Invalid token endpoint URL"),
        );
        let redirect_url = RedirectUrl::new(
            BASE_URL
                .join("/integration/finish?provider=youtube")
                .expect("Invalid redirect URL"),
        );
        let scopes = &["https://www.googleapis.com/auth/youtube.upload"];
        Oauth2Config {
            client_id,
            client_secret,
            auth_url,
            token_url,
            scopes,
            redirect_url,
        }
    }
    pub fn client(&self) -> BasicClient {
        let Oauth2Config {
            client_id,
            client_secret,
            auth_url,
            token_url,
            scopes,
            redirect_url,
        } = self;
        let client = BasicClient::new(
            client_id.clone(),
            Some(client_secret.clone()),
            auth_url.clone(),
            Some(token_url.clone()),
        ).set_redirect_url(redirect_url.clone());
        self.scopes.iter().fold(client, |client, scope| {
            client.add_scope(Scope::new(scope.to_string()))
        })
    }
}

#[derive(Debug)]
pub enum Oauth2Provider {
    Dropbox,
    YouTube,
}

impl Oauth2Provider {
    pub fn providers() -> &'static [Oauth2Provider] {
        static VARIANTS: &'static [Oauth2Provider] =
            &[Oauth2Provider::Dropbox, Oauth2Provider::YouTube];
        VARIANTS
    }

    pub fn name(&self) -> &'static str {
        match self {
            Oauth2Provider::Dropbox => "dropbox",
            Oauth2Provider::YouTube => "youtube",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Oauth2Provider::Dropbox => "Dropbox",
            Oauth2Provider::YouTube => "YouTube",
        }
    }

    pub fn client(&self) -> BasicClient {
        let config: &Oauth2Config = match self {
            Oauth2Provider::Dropbox => &*DROPBOX_CONFIG,
            Oauth2Provider::YouTube => &*YOUTUBE_CONFIG,
        };

        config.client()
    }
}

impl<'v> FromFormValue<'v> for Oauth2Provider {
    type Error = String;

    fn from_form_value(form_value: &'v RawStr) -> Result<Oauth2Provider, Self::Error> {
        let decoded = form_value.url_decode();
        match decoded {
            Ok(ref provider) if provider == "dropbox" => Ok(Oauth2Provider::Dropbox),
            Ok(ref provider) if provider == "youtube" => Ok(Oauth2Provider::YouTube),
            _ => Err(format!("unknown provider {}", form_value)),
        }
    }
}