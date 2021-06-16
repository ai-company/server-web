use actix_web::{web, HttpResponse, Responder};

use crate::database::{user::UserID, SqlPool};

async fn subscription_status(
    user_id: UserID,
    pool: SqlPool,
) -> Result<bool, Box<dyn std::error::Error>> {
    if super::stripe::status::verify_stripe_subscription(user_id, pool).await? {
        Ok(true)
    } else {
        // Note more payment providers will be inserted above, so this isnt just if something == true { true } else { false }
        Ok(false)
    }
}

// wrappers

// TODO: no authorization check?
pub async fn get(user_id: web::Path<String>, pool: web::Data<SqlPool>) -> impl Responder {
    match user_id.parse() {
        Ok(user_id) => match subscription_status(user_id, pool.as_ref().clone()).await {
            Ok(status) => match status {
                true => HttpResponse::Ok().finish(),
                false => HttpResponse::NotFound().finish(),
            },
            Err(e) => {
                println!(
                    "Failed to check Stripe subscription status with error: {}",
                    e
                );
                HttpResponse::InternalServerError().finish()
            }
        },
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}
