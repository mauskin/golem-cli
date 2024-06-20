use std::path::Path;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use golem_cli::cloud::{AuthSecret, CloudAuthenticationConfig, CloudAuthenticationConfigData};
use golem_cloud_client::model::{OAuth2Data, Token, TokenSecret, UnsafeToken};
use indoc::printdoc;
use tracing::warn;
use uuid::Uuid;

use crate::cloud::clients::login::LoginClient;
use crate::cloud::clients::CloudAuthentication;
use golem_cli::config::{CloudProfile, Config, Profile, ProfileName};
use golem_cli::model::GolemError;

#[async_trait]
pub trait Auth {
    async fn authenticate(
        &self,
        manual_token: Option<Uuid>,
        profile_name: &ProfileName,
        profile: &CloudProfile,
        config_dir: &Path,
    ) -> Result<CloudAuthentication, GolemError>;
}

pub struct AuthLive {
    pub login: Box<dyn LoginClient + Send + Sync>,
}

impl From<&CloudAuthenticationConfig> for CloudAuthentication {
    fn from(val: &CloudAuthenticationConfig) -> Self {
        CloudAuthentication(UnsafeToken {
            data: Token {
                id: val.data.id,
                account_id: val.data.account_id.to_string(),
                created_at: val.data.created_at,
                expires_at: val.data.expires_at,
            },
            secret: TokenSecret {
                value: val.secret.0,
            },
        })
    }
}

pub fn unsafe_token_to_auth_config(value: &UnsafeToken) -> CloudAuthenticationConfig {
    CloudAuthenticationConfig {
        data: CloudAuthenticationConfigData {
            id: value.data.id,
            account_id: value.data.account_id.to_string(),
            created_at: value.data.created_at,
            expires_at: value.data.expires_at,
        },
        secret: AuthSecret(value.secret.value),
    }
}

impl AuthLive {
    fn save_auth_unsafe(
        &self,
        token: &UnsafeToken,
        profile_name: &ProfileName,
        config_dir: &Path,
    ) -> Result<(), GolemError> {
        let profile = Config::get_profile(profile_name, config_dir).ok_or(GolemError(format!(
            "Can't find profile {profile_name} in config"
        )))?;

        match profile {
            Profile::Golem(_) => Err(GolemError(format!(
                "Profile {profile_name} is an OOS profile. Cloud profile expected."
            ))),
            Profile::GolemCloud(mut profile) => {
                profile.auth = Some(unsafe_token_to_auth_config(token));
                Config::set_profile(
                    profile_name.clone(),
                    Profile::GolemCloud(profile),
                    config_dir,
                )?;

                Ok(())
            }
        }
    }

    fn save_auth(&self, token: &UnsafeToken, profile_name: &ProfileName, config_dir: &Path) {
        match self.save_auth_unsafe(token, profile_name, config_dir) {
            Ok(_) => {}
            Err(err) => {
                warn!("Failed to save auth data: {err}")
            }
        }
    }

    async fn oauth2(
        &self,
        profile_name: &ProfileName,
        config_dir: &Path,
    ) -> Result<CloudAuthentication, GolemError> {
        let data = self.login.start_oauth2().await?;
        inform_user(&data);
        let token = self.login.complete_oauth2(data.encoded_session).await?;
        self.save_auth(&token, profile_name, config_dir);
        Ok(CloudAuthentication(token))
    }

    async fn profile_authentication(
        &self,
        profile_name: &ProfileName,
        profile: &CloudProfile,
        config_dir: &Path,
    ) -> Result<CloudAuthentication, GolemError> {
        if let Some(data) = &profile.auth {
            Ok(data.into())
        } else {
            self.oauth2(profile_name, config_dir).await
        }
    }
}

fn inform_user(data: &OAuth2Data) {
    let box_url_line = String::from_utf8(vec![b'-'; data.url.len() + 2]).unwrap();
    let box_code_line = String::from_utf8(vec![b'-'; data.user_code.len() + 2]).unwrap();
    let expires: DateTime<Utc> = data.expires;
    let expires_in = expires.signed_duration_since(Utc::now()).num_minutes();
    let expires_at = expires.format("%T");
    let url = &data.url;
    let user_code = &data.user_code;

    printdoc! {"
        >>
        >>  Application requests to perform OAuth2
        >>  authorization.
        >>
        >>  Visit following URL in a browser:
        >>
        >>   ┏{box_url_line}┓
        >>   ┃ {url} ┃
        >>   ┗{box_url_line}┛
        >>
        >>  And enter following code:
        >>
        >>   ┏{box_code_line}┓
        >>   ┃ {user_code} ┃
        >>   ┗{box_code_line}┛
        >>
        >>  Code will expire in {expires_in} minutes at {expires_at}.
        >>
        Waiting...
    "}
}

#[async_trait]
impl Auth for AuthLive {
    async fn authenticate(
        &self,
        manual_token: Option<Uuid>,
        profile_name: &ProfileName,
        profile: &CloudProfile,
        config_dir: &Path,
    ) -> Result<CloudAuthentication, GolemError> {
        if let Some(manual_token) = manual_token {
            let secret = TokenSecret {
                value: manual_token,
            };
            let data = self.login.token_details(secret.clone()).await?;

            Ok(CloudAuthentication(UnsafeToken { data, secret }))
        } else {
            self.profile_authentication(profile_name, profile, config_dir)
                .await
        }
    }
}
