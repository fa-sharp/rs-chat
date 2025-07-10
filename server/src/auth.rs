mod guard;
mod oauth;
mod session;
mod sso_header;

pub use guard::ChatRsUserId;
pub use oauth::{setup_oauth, DiscordOAuthConfig, GitHubOAuthConfig, GoogleOAuthConfig};
pub use session::{setup_session, ChatRsAuthSession};
pub use sso_header::{setup_sso_header_auth, SSOHeaderMergedConfig};

use rocket::fairing::AdHoc;

use crate::{config::get_app_config, utils::encryption::Encryptor};

/// Fairing that sets up an encryption service
pub fn setup_encryption() -> AdHoc {
    AdHoc::on_ignite("Encryption setup", |rocket| async {
        let app_config = get_app_config(&rocket);
        let encryptor = Encryptor::new(&app_config.secret_key)
            .expect("Invalid secret key: must be 64-character hexadecimal string");

        rocket.manage(encryptor)
    })
}
