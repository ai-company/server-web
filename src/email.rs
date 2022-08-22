use lettre::smtp::authentication::IntoCredentials;
use lettre::smtp::error::SmtpResult;
use lettre::{SmtpClient, Transport};
use lettre_email::{EmailBuilder, Mailbox};
use std::env;

pub fn email_send(to: &str, subject: &str, content: &str) -> SmtpResult {
    let smtp_address = env::var("MAIL_HOST").unwrap_or("127.0.0.1".into());
    let username = env::var("MAIL_USER").unwrap_or("".into());
    let password = env::var("MAIL_PASS").unwrap_or("".into());

    let built_email = EmailBuilder::new()
        .to(to)
        .from(Mailbox::new_with_name(
            "mail.orto.ai".into(),
            "noreply@mail.orto.ai".into(),
        ))
        .subject(subject)
        .html(content)
        .build()
        .unwrap()
        .into();

    let credentials = (username, password).into_credentials();
    let mut client = SmtpClient::new_simple(&smtp_address)?
        .credentials(credentials)
        .transport();

    client.send(built_email)
}

pub fn email_send_all(to: &[&str], subject: &str, content: &str) -> SmtpResult {
    let smtp_address = env::var("MAIL_HOST").unwrap_or("127.0.0.1".into());
    let username = env::var("MAIL_USER").unwrap_or("".into());
    let password = env::var("MAIL_PASS").unwrap_or("".into());

    let mut email = EmailBuilder::new();

    for address in to {
        email = email.bcc(*address);
    }

    let built_email = email
        .to("noreply@mail.orto.ai")
        .from(Mailbox::new_with_name(
            "mail.orto.ai".into(),
            "noreply@mail.orto.ai".into(),
        ))
        .subject(subject)
        .html(content)
        .build()
        .unwrap()
        .into();

    let credentials = (username, password).into_credentials();
    let mut client = SmtpClient::new_simple(&smtp_address)?
        .credentials(credentials)
        .transport();

    client.send(built_email)
}
