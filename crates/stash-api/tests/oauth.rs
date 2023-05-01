use stash_api::common::poe_api::get_oauth_token;

#[tokio::test]
async fn test_oauth() {
    let client_id = std::env::var("POE_CLIENT_ID").expect("POE_CLIENT_ID");
    let client_secret = std::env::var("POE_CLIENT_SECRET").expect("POE_CLIENT_SECRET");
    get_oauth_token(&client_id, &client_secret)
        .await
        .expect("fetching OAuth token");
}
