use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::{self, BufRead};
use std::path::Path;

use crypto::digest::Digest;
use crypto::sha2::Sha256;

use lettre::smtp::authentication::IntoCredentials;
use lettre::{SmtpClient, Transport};
use lettre_email::EmailBuilder;

use nanoid;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

pub fn token_id_for_mail() -> String {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("db/tokenids.txt")
        .unwrap();

    let id = nanoid::simple();

    if let Err(e) = writeln!(file, "{}\n", id) {
        eprintln!("Couldn't write to file: {}", e);
    }

    id
}

pub fn sell_token(id: &str) -> Option<String> {
    if id.trim().is_empty() {
        return None;
    }

    if let Ok(lines) = read_lines("db/tokenids.txt") {
        let lines = lines.map(|x| x.unwrap()).collect::<Vec<String>>();

        for (_, line) in lines.iter().enumerate() {
            if line.trim() == id {
                let token = nanoid::simple();

                let mut file = OpenOptions::new()
                    .write(true)
                    .append(true)
                    .open("db/tokens.txt")
                    .unwrap();

                if let Err(e) = writeln!(file, "{}", token) {
                    eprintln!("Couldn't write to file: {}", e);
                }

                return Some(token);
            }
        }
    }

    None
}

pub fn is_invited(token: &str) -> bool {
    if let Ok(lines) = read_lines("db/tokens.txt") {
        for line in lines {
            if let Ok(ip) = line {
                if ip.trim() == token {
                    return true;
                }
            }
        }
    }

    false
}

pub fn try_send_token_to(email: &str) -> Result<(), ()> {
    let mut hasher = Sha256::new();

    hasher.input_str(&email);

    let email_hash = hasher.result_str();

    if let Ok(lines) = read_lines("db/emails.txt") {
        let mut found = false;

        for line in lines {
            if let Ok(ip) = line {
                if ip.trim() == email_hash {
                    found = true;
                    break;
                }
            }
        }

        if !found {
            return Err(());
        }

        let token = token_id_for_mail();

        let smtp_address = "smtp.gmail.com";
        let username = "";
        let password = "";
        let built_email = EmailBuilder::new()
            .to(email)
            .from(username)
            .subject("Adgang til testing af staveværktøj.")
            .text(format!(
                "Velkommen til Orto AI's lukkede alfa-test!\n\nKlik på linket for at få adgang til Orto: https://orto.ai/editor?token={}\n\nLinket gemmer en nøgle til siden i den browser, og det vil kræve et nyt link for hver browser.\nTusind tak for at deltage. Vi ser frem til din feedback.",
                token
            ))
            .build()
            .unwrap()
            .into();

        let credentials = (username, password).into_credentials();
        let mut client = SmtpClient::new_simple(smtp_address)
            .unwrap()
            .credentials(credentials)
            .transport();

        let _result = client.send(built_email);

        Ok(())
    } else {
        Err(())
    }
}
