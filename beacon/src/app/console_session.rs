use chrono::Duration;
use dropshot::{HttpError, HttpResponseCreated, HttpResponseHeaders, HttpResponseOk};
use http::header;
use lucid_auth::authn::external::session_cookie::session_cookie_header_value;
use lucid_common::api::error::Error;
use lucid_types::{dto::{params, views}, identity::Resource};
use lucid_uuid_kinds::{GenericUuid, OrganisationIdUuid};
use rand::{RngCore, SeedableRng, rngs::StdRng};

use crate::app::Beacon;

impl Beacon {
    pub async fn console_session_start(
        &self,
        params: params::LoginParams,
    ) -> Result<HttpResponseOk<views::LoginResponse>, Error> {
        let user = self
            .datastore
            .user_get_by_email(
                &self.opctx_external_authn,
                &params.email.trim()
            )
            .await?
            .ok_or_else(|| Error::Unauthenticated {
                internal_message: format!("no user with email: {}", params.email),
            })?;

        let user_id = Resource::id(&user);

        let valid = self
            .datastore
            .user_verify_password(
                &self.opctx_external_authn,
                user_id,
                &params.password
            )
            .await?;

        if !valid {
            return Err(Error::Unauthenticated {
                internal_message: "invalid password".into(),
            }
            .into());
        }

        let orgs = self
            .datastore
            .organisations_for_user(user_id)
            .await
            .map_err(|e| {
                Error::internal_anyhow("failed to list organisations".into(), e)
            })?;

        let organisations = orgs
            .into_iter()
            .map(|org| {
                let org_id = lucid_types::identity::Resource::id(&org);
                views::LoginOrganisation {
                    id: org_id.into_untyped_uuid(),
                    name: org.name,
                    display_name: org.display_name,
                }
            })
            .collect();

        Ok(HttpResponseOk(views::LoginResponse { organisations }))
    }

    pub async fn console_session_login(
        &self,
        params: params::LoginSessionParams,
    ) -> Result<HttpResponseHeaders<HttpResponseCreated<()>>, HttpError> {
        // --- re-validate credentials ----------------------------------------

        let user = self
            .datastore
            .user_get_by_email(&self.opctx_external_authn, &params.email)
            .await?
            .ok_or_else(|| Error::Unauthenticated {
                internal_message: format!("no user with email: {}", params.email),
            })?;

        let user_id = lucid_types::identity::Resource::id(&user);

        let valid = self
            .datastore
            .user_verify_password(
                &self.opctx_external_authn,
                user_id,
                &params.password
            )
            .await?;

        if !valid {
            return Err(Error::Unauthenticated {
                internal_message: "invalid password".into(),
            }
            .into());
        }

        // --- verify org membership ------------------------------------------

        let organisation_id =
            OrganisationIdUuid::from_untyped_uuid(params.organisation_id);

        let orgs = self
            .datastore
            .organisations_for_user(user_id)
            .await
            .map_err(|e| {
                Error::internal_anyhow("failed to list organisations".into(), e)
            })?;

        let is_member = orgs.iter().any(|org| {
            let org_id: OrganisationIdUuid = lucid_types::identity::Resource::id(org);
            org_id == organisation_id
        });

        if !is_member {
            return Err(Error::Forbidden.into());
        }

        // --- create session -------------------------------------------------

        let token = gen_session_id();

        self.datastore
            .session_create(user_id, organisation_id, &token)
            .await
            .map_err(|e| {
                Error::internal_anyhow("failed to create session".into(), e)
            })?;

        // --- set cookie -----------------------------------------------------

        let cookie_value = session_cookie_header_value(
            &token,
            Duration::new(self.config.session.idle_timeout_seconds, 0).ok_or(Error::Forbidden)?,
            self.config.session.secure,
        )?;

        let mut response = HttpResponseHeaders::new_unnamed(HttpResponseCreated(()));
        response
            .headers_mut()
            .insert(header::SET_COOKIE, cookie_value);

        Ok(response)
    }
}

pub(crate) fn gen_session_id() -> String {
    let mut rng = StdRng::from_os_rng();
    let mut random_bytes: [u8; 20] = [0; 20];
    rng.fill_bytes(&mut random_bytes);
    hex::encode(random_bytes)
}
