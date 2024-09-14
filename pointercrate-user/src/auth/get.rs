use std::collections::HashSet;

use crate::{
    auth::{AccessClaims, AuthenticatedUser},
    error::Result,
    User,
};
use jsonwebtoken::{DecodingKey, Validation};
use log::{debug, info};
use pointercrate_core::error::CoreError;
use sqlx::{Error, PgConnection};

impl AuthenticatedUser {
    pub async fn token_auth(access_token: &str, csrf_token: Option<&str>, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        info!("We are expected to perform token authentication");

        // Well this is reassuring. Also we directly deconstruct it and only save the ID
        // so we don't accidentally use unsafe values later on
        let mut no_validation = Validation::default();
        no_validation.insecure_disable_signature_validation();
        no_validation.validate_exp = false;
        no_validation.required_spec_claims = HashSet::new();

        let AccessClaims { id, .. } = jsonwebtoken::decode(access_token, &DecodingKey::from_secret(b""), &no_validation)
            .map_err(|_| CoreError::Unauthorized)?
            .claims;

        debug!("The token identified the user with id {}, validating...", id);

        // Note that at this point we haven't validated the access token OR the csrf token yet.
        // However, the key they are signed with encompasses the password salt for the user they supposedly
        // identify, so we need to retrieve that.
        let user = Self::by_id(id, connection).await?.validate_access_token(access_token)?;

        if let Some(csrf_token) = csrf_token {
            user.validate_csrf_token(csrf_token)?
        }

        Ok(user)
    }

    pub(in crate::auth) async fn by_id(id: i32, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, email_address::text, password_hash, google_account_id FROM members WHERE member_id = $1"#,
            id
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => {
                if row.google_account_id.is_some() && row.email_address.is_some() {
                    Ok(AuthenticatedUser::oauth2(
                        construct_from_row!(row),
                        row.email_address.unwrap(),
                        row.google_account_id.unwrap(),
                    ))
                } else {
                    Ok(AuthenticatedUser::legacy(
                        construct_from_row!(row),
                        row.password_hash.expect("password hash is null"),
                    ))
                }
            },
        }
    }

    pub(in crate::auth) async fn by_name(name: &str, connection: &mut PgConnection) -> Result<AuthenticatedUser> {
        let row = sqlx::query!(
            r#"SELECT member_id, members.name, permissions::integer, display_name, youtube_channel::text, email_address::text, password_hash, google_account_id FROM members WHERE members.name = $1"#,
            name.to_string()
        )
        .fetch_one(connection)
        .await;

        match row {
            Err(Error::RowNotFound) => Err(CoreError::Unauthorized.into()),
            Err(err) => Err(err.into()),
            Ok(row) => {
                if row.google_account_id.is_some() && row.email_address.is_some() {
                    Ok(AuthenticatedUser::oauth2(
                        construct_from_row!(row),
                        row.email_address.unwrap(),
                        row.google_account_id.unwrap(),
                    ))
                } else {
                    Ok(AuthenticatedUser::legacy(
                        construct_from_row!(row),
                        row.password_hash.expect("password hash is null"),
                    ))
                }
            },
        }
    }
}
