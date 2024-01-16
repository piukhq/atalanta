//! a simple mock hermes to serve payment card user info requests based on the tokens file.
use std::{collections::HashMap, fs::File, path::Path, sync::Arc};

use atalanta::configuration::load_settings;
use axum::{extract::State, routing::post, Json, Router};
use color_eyre::Result;
use eyre::Context;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize)]
struct CardInfo {
    first_six: String,
    last_four: String,
}

#[derive(Hash, Eq, PartialEq)]
struct Token(String);

struct AppState {
    user_info: HashMap<Token, CardInfo>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let settings = load_settings()?;
    let user_info = load_payment_card_user_info(settings.tokens_file_path)?;

    let app = Router::new()
        .route(
            "/payment_cards/accounts/payment_card_user_info/:retailer_slug",
            post(payment_card_user_info),
        )
        .with_state(Arc::new(AppState { user_info }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8002").await?;
    info!(
        addr = listener.local_addr()?.to_string(),
        "Starting listener."
    );

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Deserialize)]
struct PaymentCardUserInfoRequest {
    payment_cards: Vec<String>,
}

#[derive(Serialize)]
struct PaymentCardUserInfoResponse {
    loyalty_id: String,
    scheme_account_id: usize,
    payment_card_account_id: usize,
    user_id: usize,
    credentials: String,
    card_information: CardInformation,
}

#[derive(Serialize)]
struct CardInformation {
    first_six: String,
    last_four: String,
    expiry_month: u8,
    expiry_year: u16,
}

fn find_card_info<'a>(
    user_info: &'a HashMap<Token, CardInfo>,
    token: &str,
) -> Option<&'a CardInfo> {
    user_info
        .iter()
        .find(|(token_b, _)| token == token_b.0)
        .map(|(_, card_info)| card_info)
}

fn make_payment_card_user_info_response(
    card_info: &CardInfo,
    id: usize,
) -> PaymentCardUserInfoResponse {
    PaymentCardUserInfoResponse {
        loyalty_id: String::from(""),
        scheme_account_id: id,
        payment_card_account_id: id,
        user_id: id,
        credentials: String::from(""),
        card_information: CardInformation {
            first_six: card_info.first_six.clone(),
            last_four: card_info.last_four.clone(),
            expiry_month: 0,
            expiry_year: 0,
        },
    }
}

async fn payment_card_user_info(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<PaymentCardUserInfoRequest>,
) -> Json<HashMap<String, PaymentCardUserInfoResponse>> {
    info!(
        count = payload.payment_cards.len(),
        "Received payment card user info request."
    );
    let result: HashMap<String, PaymentCardUserInfoResponse> = payload
        .payment_cards
        .iter()
        .enumerate()
        .flat_map(|(idx, token)| {
            find_card_info(&state.user_info, token).map(|card_info| {
                (
                    token.clone(),
                    make_payment_card_user_info_response(card_info, idx),
                )
            })
        })
        .collect();

    info!(
        identified = result.len(),
        total = payload.payment_cards.len(),
        "Returning identified card info."
    );

    Json(result)
}

fn load_payment_card_user_info<P>(path: P) -> Result<HashMap<Token, CardInfo>>
where
    P: AsRef<Path>,
{
    // Load token and slugs derived from the Hermes database
    let file = File::open(&path)
        .wrap_err_with(|| format!("Failed to open {}", path.as_ref().display()))?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(file);

    #[derive(Deserialize)]
    struct Record {
        token: String,
        _retailer_slug: String,
        first_six: String,
        last_four: String,
        _payment_slug: String,
    }

    let records = rdr
        .deserialize()
        .collect::<Result<Vec<Record>, csv::Error>>()?;
    Ok(records
        .into_iter()
        .map(|r| {
            (
                Token(r.token),
                CardInfo {
                    first_six: r.first_six,
                    last_four: r.last_four,
                },
            )
        })
        .collect())
}
