use color_eyre::Result;
use rand::Rng;

/// Inserts data into writer and writes to a file
///
/// # Error
///
/// If an error occurs, the error is returned to `main`
fn write_to_file(path: &str) -> Result<()> {
    // Creates new `Writer` for `stdout`
    let mut writer = csv::Writer::from_path(path)?;
    let merchant_slugs = [
        "wasabi-club",
        "iceland-bonus-card",
        "bpl-viator",
        "squaremeal",
    ];
    let payment_slugs = ["visa", "amex", "mastercard"];

    // We don't explicitly write the header record
    for n in 1..=1_000_000 {
        let merch_range = rand::thread_rng().gen_range(0..4);
        let merchant_slug: String = merchant_slugs[merch_range].to_owned();
        let pay_range = rand::thread_rng().gen_range(0..3);
        let payment_slug = payment_slugs[pay_range].to_owned();
        let token = format!("token_{n}");
        let first_six = rand::thread_rng().gen_range(100_000..999_999).to_string();
        let last_four = rand::thread_rng().gen_range(1000..9999).to_string();

        writer.write_record(&[token, merchant_slug, payment_slug, first_six, last_four])?;
    }

    writer.flush()?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    write_to_file("./files/hermes_tokens.csv")?;

    Ok(())
}
