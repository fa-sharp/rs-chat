use std::path::{Path, PathBuf};

use rocket::{
    fairing::AdHoc,
    fs::{relative, FileServer, NamedFile},
    get,
    http::Header,
    routes, Responder, State,
};

use crate::config::{get_app_config, AppConfig};

const WEB_DIST: &str = relative!("../web/dist");

pub fn setup_static_files() -> AdHoc {
    AdHoc::on_ignite("Static files", |rocket| async {
        let app_config = get_app_config(&rocket);
        let static_path = PathBuf::from(app_config.static_path.as_deref().unwrap_or(WEB_DIST));

        rocket
            .mount("/", FileServer::from(static_path).rank(1))
            .mount("/", routes![wildcard])
    })
}

#[derive(Responder)]
struct WildcardResponse {
    inner: NamedFile,
    cache_control: Header<'static>,
}

/// Wildcard route handler for client-side routing.
#[get("/<_..>", rank = 10)]
async fn wildcard(app_config: &State<AppConfig>) -> Option<WildcardResponse> {
    let static_path = app_config.static_path.as_deref().unwrap_or(WEB_DIST);
    let index_html_path = Path::new(static_path).join("index.html");

    Some(WildcardResponse {
        inner: NamedFile::open(index_html_path).await.ok()?,
        cache_control: Header::new("Cache-Control", "public,max-age=0,must-revalidate"),
    })
}
