use time::Duration;
use tower_cookies::Cookie;

pub struct CookieHelper;

impl CookieHelper {
    pub const SESSION_COOKIE_NAME: &'static str = "session_id";

    pub fn create_session_cookie(session_id: String, max_age_minutes: i64) -> Cookie<'static> {
        let mut cookie = Cookie::new(Self::SESSION_COOKIE_NAME, session_id);
        cookie.set_path("/");
        cookie.set_http_only(true);
        cookie.set_same_site(tower_cookies::cookie::SameSite::Strict);
        cookie.set_secure(true);
        cookie.set_max_age(Duration::minutes(max_age_minutes));
        cookie
    }
}
