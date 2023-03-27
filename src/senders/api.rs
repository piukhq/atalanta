use super::Sender;
use color_eyre::Result;

pub struct APISender {
    pub url: String,
}

impl Sender for APISender {
    fn send(&self, transactions: String) -> Result<()> {
        let client = reqwest::blocking::Client::new();
        println!("{:?}", transactions);

        let resp = client.post(&self.url).body(transactions).send()?;
        println!("{}", resp.status());

        Ok(())
    }
}
