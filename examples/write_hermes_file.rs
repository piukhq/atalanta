use color_eyre::Result;
use csv;
use rand::Rng;

/// Inserts data into writer and writes to a file
///
/// # Error
///
/// If an error occurs, the error is returned to `main`
fn write_to_file(path: &str) -> Result<()>{
    // Creates new `Writer` for `stdout`
    let mut writer = csv::Writer::from_path(path)?;
    let merchant_slugs = vec!["wasabi-club", "iceland-bonus-card", "viator", "squaremeal"];
    let payment_slugs = vec!["visa", "amex", "mastercard"];

    // We don't explicitly write the header record
    for n in 1..1000000 {
        let merch_range = rand::thread_rng().gen_range(0..4);
        let merchant_slug:String = merchant_slugs[merch_range].to_string();
        let pay_range = rand::thread_rng().gen_range(0..3);
        let payment_slug:String = payment_slugs[pay_range].to_string();
        let token = format!("token_{}", n);
        writer.write_record(&[token, merchant_slug, payment_slug])?;
    };

    writer.flush()?;

    Ok(())
}

fn main() -> Result<()>{
    color_eyre::install()?;
    
    write_to_file("./files/hermes_tokens.csv")?;

    Ok(())
}
