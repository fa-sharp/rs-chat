use chat_rs_api::{build_rocket, utils::setup_json_logging};
use rocket::launch;

#[launch]
pub fn rocket() -> _ {
    if cfg!(not(debug_assertions)) {
        setup_json_logging();
    }

    build_rocket()
}
