use color_eyre::Result;
use eyre::OptionExt;
use std::env;
use std::ffi::OsString;
use std::fs::File;
use std::time;

fn run() -> Result<()> {
    let mut tokens = Vec::new();
    let query = env::args().nth(2).unwrap_or_default();

    let file_path = get_first_arg()?;
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    let start = time::Instant::now();
    for result in rdr.records() {
        let record = result?;
        if record.iter().any(|field| field == query) {
            tokens.push(record);
        }
    }
    let duration = start.elapsed();
    println!("duration = {duration:?}");
    println!("Number = {}", tokens.len());

    Ok(())
}

/// Returns the first positional argument sent to this process. If there are no
/// positional arguments, then this returns an error.
fn get_first_arg() -> Result<OsString> {
    env::args_os()
        .nth(1)
        .ok_or_eyre("expected at least one argument")
}

fn main() -> Result<()> {
    color_eyre::install()?;

    run()?;

    Ok(())
}
