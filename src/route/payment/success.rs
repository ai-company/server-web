use actix_web::{
    http::header,
    web::{self, Data},
    HttpResponse, Responder,
};

use handlebars::Handlebars;

use crate::{
    database::{
        self, signup_tokens,
        stripe_profile::{self},
        user, SqlPool, SyncTransaction,
    },
    middleware,
    route::payment::stripe::status::verify_stripe_subscription,
};

//

pub async fn get(
    template: Data<Handlebars<'_>>,
    session: middleware::context::SessionContext,
    pool: web::Data<SqlPool>,
    _: middleware::UserAuthorized,
) -> impl Responder {
    if let middleware::Session::Guest = session.user {
        HttpResponse::Found().header(header::LOCATION, "/").finish()
    } else {
        let user = session.user.unwrap_user();

        let subscription = verify_stripe_subscription(user.id, &pool)
            .await
            .unwrap()
            .unwrap();

        let stripe = match database::stripe_profile::get(user.id, pool.as_ref()) {
            Ok(Some(v)) => v,
            Err(e) => {
                println!("Failed to get verify stripe subscription with error: {}", e);
                unreachable!();
            }
            _ => unreachable!(),
        };

        stripe_profile::set_subscription(
            &stripe.stripe_id,
            subscription.current_period_start,
            subscription.current_period_end,
            &pool,
        )
        .unwrap();

        // registration and payment successful, drop the signup token
        let trans = SyncTransaction::new(&pool).unwrap();
        signup_tokens::delete(user.id, &trans).unwrap();
        trans.commit().unwrap();

        user::verify(user, &pool).unwrap();

        HttpResponse::Ok().body(template.render("page/payment/success", &session).unwrap())
    }
}
