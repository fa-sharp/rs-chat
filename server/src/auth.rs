mod api_key;
mod guard;
mod oauth;
mod session;
mod sso_header;

use rocket::fairing::AdHoc;

pub use api_key::build_api_key_string;
pub use guard::ChatRsUserId;
pub use oauth::{DiscordOAuthConfig, GitHubOAuthConfig, GoogleOAuthConfig, OIDCConfig};
pub use session::ChatRsAuthSession;
pub use sso_header::SSOHeaderMergedConfig;
use {oauth::setup_oauth, session::setup_session, sso_header::setup_sso_header_auth};

/// Fairing that sets up all authentication services
pub fn setup_auth(base_path: &str) -> AdHoc {
    let base_path = base_path.to_owned();

    AdHoc::on_ignite("Auth services", |rocket| async {
        rocket
            .attach(setup_session())
            .attach(setup_sso_header_auth())
            .attach(setup_oauth(base_path))
    })
}
