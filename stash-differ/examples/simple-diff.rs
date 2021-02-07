use stash_differ::{SampleImporter, StashDiffer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stash_records = SampleImporter::from_file("./sample_raw.csv");

    stash_records
        .iter()
        .zip(stash_records.iter().skip(1))
        .for_each(|(before, after)| {
            let events = StashDiffer::diff(&before, &after);
            dbg!(events);
        });

    Ok(())
}
