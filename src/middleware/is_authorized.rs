use crate::tokens::is_invited;

use cookie::Cookie;
use hyper::{header, Body, Request};

pub fn is_authorized(req: &Request<Body>) -> bool {
    if let Some(cookies) = req.headers().get(header::COOKIE) {
        for cookie in cookies.to_str().unwrap().split(';') {
            let cookie = Cookie::parse_encoded(cookie).unwrap();

            if cookie.name() == "auth" && is_invited(cookie.value()) {
                return true;
            }
        }
    }

    return false;
}
