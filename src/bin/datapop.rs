use color_eyre::Result;
use eyre::Context;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

const BINK_CLIENT_ID: &str = "MKd3FfDGBi1CIUQwtahmPap64lneCa2R6GvVWKg6dNg4w9Jnpd";

#[derive(Deserialize)]
struct Record {
    token: String,
    retailer_slug: String,
    first_six: String,
    last_four: String,
    payment_slug: String,
}

struct User {
    id: usize,
    email: String,
    uid: String,
}

fn main() -> color_eyre::Result<()> {
    let records = load_payment_card_user_info("files/hermes_tokens.csv")?;

    let users = records.iter().enumerate().map(|(i, _record)| User {
        id: i,
        email: format!("user{i}@testbink.com"),
        uid: format!("uid{i}"),
    });

    println!("COPY public.user (id, password, is_superuser, email, is_active, date_joined, is_staff, uid, client_id, salt, external_id, is_tester, delete_token, bundle_id) FROM stdin;");
    for user in users {
        let fields = [
            &user.id.to_string(),       // id
            "password",                 // password
            "false",                    // is superuser
            &user.email,                // email
            "true",                     // is active
            "2020-03-09 12:42:15+0000", // date joined
            "false",                    // is staff
            &user.uid,                  // uid
            BINK_CLIENT_ID,             // client_id
            "abcdefgh",                 // salt
            &user.email,                // external id
            "false",                    // is tester
            "",                         // delete token
            "com.bink.wallet",          // bundle id
        ];

        println!("{}", fields.join("\t"));
    }
    println!(r"\.");

    Ok(())
}

fn load_payment_card_user_info<P>(path: P) -> Result<Vec<Record>>
where
    P: AsRef<Path>,
{
    // Load token and slugs derived from the Hermes database
    let file = File::open(&path)
        .wrap_err_with(|| format!("Failed to open {}", path.as_ref().display()))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    Ok(rdr
        .deserialize()
        .collect::<Result<Vec<Record>, csv::Error>>()?)
}
