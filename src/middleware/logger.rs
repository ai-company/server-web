use std::future::Future;

use actix_web::dev::{Service, ServiceRequest, ServiceResponse};
use chrono::Utc;

/// Scope request logging middleware
///
/// This logs the executed request with format: `dd/mm/yy hh:mm:ss REQTIMEµs METHOD STATUS PATH`
pub fn logger(
    req: ServiceRequest,
    srv: &mut impl Service<
        Request = ServiceRequest,
        Response = ServiceResponse,
        Error = actix_web::Error,
    >,
) -> impl Future<Output = Result<ServiceResponse, actix_web::Error>> {
    let start_time = Utc::now().time();
    let date = Utc::now().format("%d/%m/%y %T").to_string();
    let method = req.method().as_str().to_owned();
    let path = req.uri().path().to_owned();

    let fut = srv.call(req);

    async move {
        let result = fut.await;

        match &result {
            Ok(response) => {
                let end_time = Utc::now().time();
                let time = end_time - start_time;

                let color = if response.status().as_u16() >= 400 {
                    "\x1b[31m"
                } else if response.status().as_u16() == 304 {
                    ""
                } else {
                    "\x1b[32m"
                };

                println!(
                    "{}{} {:>6}µs {:>7} {} {:.128}\x1b[0m",
                    color,
                    date,
                    time.num_microseconds().unwrap(),
                    method,
                    response.status().as_u16(),
                    path
                );
            }
            Err(_) => println!("{} \x1b[41m{:7} --- {}\x1b[0m", date, method, path),
        }

        result
    }
}
