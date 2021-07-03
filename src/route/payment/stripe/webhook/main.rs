use actix_web::{
    web::{self, Json},
    HttpRequest, HttpResponse, Responder,
};
// use hmac::{Hmac, Mac, NewMac};

use super::{auth::authenticate, Event, EventType};
use crate::{
    database::SqlPool,
    route::{EndpointProcessingError, EndpointProcessingResult},
};

crate::basic_error_with_message!(
    WrongWebhookSignature,
    "Stripe webhook activated with invalid signature"
);

// Log the lastest stripe webhook to a file, only to be used in debugging
fn log_request(input: &str) -> std::io::Result<()> {
    use std::fs::File;
    use std::io::Write;
    let mut file = File::create("latest_request.json")?;
    file.write_all(input.as_bytes())?;
    Ok(())
}

// Function run when any webhook is recived
pub async fn process_webhook(
    req: &HttpRequest,
    body: String,
    event: Json<Event>,
    pool: &SqlPool,
) -> EndpointProcessingResult<()> {
    let headers = req.headers();

    authenticate(&headers, &body)?;

    let is_debugging_backend = cfg!(debug_assertions);
    // If we're debuggling we'll log this event
    if is_debugging_backend {
        let _ = log_request(&body);
    }

    if is_debugging_backend == event.livemode {
        if is_debugging_backend {
            crate::basic_error!(ExpectedNonLivemode, "Got request that is in livemode, but this backend build is debug so this is very bad please don't do that thx.");
            return Err(EndpointProcessingError::Processing(Box::new(
                ExpectedNonLivemode,
            )));
        } else {
            crate::basic_error!(
                ExpectedLivemode,
                "Production backend does not expect Stripe events that are not in livemode"
            );
            return Err(EndpointProcessingError::Processing(Box::new(
                ExpectedLivemode,
            )));
        }
    }

    if event.event_type.is_none() {
        println!("what");
        return Ok(());
    }

    use EventType::*;

    match event.event_type.unwrap() {
        // These types are all functionally identical, since they all just illustrate a change happened, and include internal data that describes the exact change to perform
        CustomerSubscriptionUpdated
        | CustomerSubscriptionCreated
        | CustomerSubscriptionDeleted
        | CustomerSubscriptionTrialWillEnd => {
            super::subscription_update::subscription_changed(event.data.object.clone(), pool)
        }
        _ => todo!(),
    }
}

// wrappers

pub async fn post(
    req: HttpRequest,
    body: String,
    event: Json<Event>,
    pool: web::Data<SqlPool>,
) -> impl Responder {
    use EndpointProcessingError::*;

    match process_webhook(&req, body, event, pool.as_ref()).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(Unauthorized) => HttpResponse::Unauthorized().finish(),
        Err(RequestUnparsable) | Err(RequestNonsensical) => HttpResponse::BadRequest().finish(),
        Err(Processing(e)) => {
            println!("Failed to react to webhook with error: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
