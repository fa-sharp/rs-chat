use std::net::IpAddr;

use rocket::request::{FromRequest, Outcome};

/// Session metadata extracted from request headers.
pub struct SessionMeta<'r> {
    pub ip: Option<IpAddr>,
    pub user_agent: Option<&'r str>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for SessionMeta<'r> {
    type Error = ();

    async fn from_request(req: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        Outcome::Success(SessionMeta {
            ip: req.client_ip(),
            user_agent: req.headers().get_one("User-Agent"),
        })
    }
}
