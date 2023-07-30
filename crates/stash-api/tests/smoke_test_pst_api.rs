use stash_api::{
    common::{poe_api::get_oauth_token, poe_ninja_client},
    r#async::indexer::Indexer,
};
use trade_common::secret::SecretString;

fn extract_credentials_from_env() -> (String, SecretString, SecretString) {
    let _ = dotenv::from_path("../../configuration/environments/.env.development"); // for local testing
    let client_id = std::env::var("POE_CLIENT_ID").expect("reading POE_CLIENT_ID");
    let client_secret =
        SecretString::new(std::env::var("POE_CLIENT_SECRET").expect("reading POE_CLIENT_SECRET"));
    let developer_mail =
        SecretString::new(std::env::var("POE_DEVELOPER_MAIL").expect("reading POE_CLIENT_SECRET"));
    (client_id, client_secret, developer_mail)
}

#[tokio::test]
async fn test_pst_api_oauth_async() {
    let (client_id, client_secret, developer_mail) = extract_credentials_from_env();
    get_oauth_token(&client_id, &client_secret, &developer_mail)
        .await
        .expect("fetching OAuth token");
}

#[tokio::test]
async fn test_stream_pst_api() {
    let (client_id, client_secret, developer_mail) = extract_credentials_from_env();

    let indexer = Indexer::new();
    let change_id = poe_ninja_client::PoeNinjaClient::fetch_latest_change_id_async()
        .await
        .expect("fetch latest change id");
    let mut rx = indexer
        .start_at_change_id(client_id, client_secret, developer_mail, change_id)
        .await;

    let mut counter = 0;
    while (rx.recv().await).is_some() {
        counter += 1;
        if counter == 2 {
            break;
        }
    }
}
