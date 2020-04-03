use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct StashTabResponse {
    next_change_id: String,
    stashes: Vec<Stash>,
}

#[derive(Debug, Deserialize)]
struct Stash {
    accountName: Option<String>,
    lastCharacterName: Option<String>,
    id: String,
    stash: Option<String>,
    stashType: String,
    items: Vec<Item>,
    public: bool,
    league: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Item {
    name: String,
    note: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://www.pathofexile.com/api/public-stash-tabs?id=649994034-665472248-633359039-717785899-684526645";
    // let result: StashTabResponse = ureq::get(url).call().into_json();
    let response = ureq::get(url).call().into_json().unwrap();
    let deserialized: StashTabResponse = serde_json::from_value(response).unwrap();
    // println!("{:?}", deserialized);
    Ok(())
}
