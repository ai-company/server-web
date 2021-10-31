use lettre::smtp::authentication::IntoCredentials;
use lettre::smtp::error::SmtpResult;
use lettre::{SmtpClient, Transport};
use lettre_email::{EmailBuilder, Mailbox};

pub fn email_send(to: &str, subject: &str, content: &str) -> SmtpResult {
    let smtp_address = "smtp.gmail.com";
    let username = "aicompanyautomatic@gmail.com";
    let password = "krkaciqcsadpgfyh";

    let built_email = EmailBuilder::new()
        .to(to)
        .from(Mailbox::new_with_name("orto.ai".into(), username.into()))
        .subject(subject)
        .html(content)
        .build()
        .unwrap()
        .into();

    let credentials = (username, password).into_credentials();
    let mut client = SmtpClient::new_simple(smtp_address)?
        .credentials(credentials)
        .transport();

    client.send(built_email)
}
