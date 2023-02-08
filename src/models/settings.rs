#[derive(serde::Deserialize)]
pub struct Settings {
    #[serde(default="default_environment")]
    pub environment: String,
}

fn default_environment() -> String {
    String::from("LOCAL")
}