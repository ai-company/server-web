use actix_web::http::header;
use actix_web::web;
use actix_web::{web::Data, HttpRequest, HttpResponse, Responder};
use serde_json::json;

use crate::database::{correction_reports, feedback, user, SqlPool, SyncTransaction};
use crate::helper::get_current_user;

use crate::middleware;
use crate::route::{EndpointProcessingError, EndpointProcessingResult};

use handlebars::Handlebars;

// Wrappers

pub async fn get(
    template: Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
    pool: web::Data<SqlPool>,
    _: middleware::AdminAuthorized,
) -> impl Responder {
    if let middleware::AdminSession::Guest = session.admin {
        HttpResponse::Found()
            .header(header::LOCATION, "/admin/signin")
            .finish()
    } else {
        let users = user::get_all(&pool).ok();
        let correction_reports = correction_reports::get_all(&pool).ok();
        let feedback = feedback::get_all(&pool).ok();

        HttpResponse::Ok().body(
            template
                .render(
                    "page/admin/account",
                    &json!({
                        "admin": &session.admin,
                        "user": &session.user,
                        "users": &users,
                        "correction_reports": &correction_reports,
                        "feedback": &feedback,
                    }),
                )
                .unwrap(),
        )
    }
}
