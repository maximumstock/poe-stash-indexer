use stash_differ::{DiffEvent, DiffStats, SampleImporter, StashDiffer};

fn main() {
    let stash_records = SampleImporter::from_file("./sample_raw.csv");

    let mut stats = DiffStats::default();

    stash_records
        .iter()
        .zip(stash_records.iter().skip(1))
        .for_each(|(before, after)| {
            let events = StashDiffer::diff(&before, &after);

            if !events.is_empty() {
                events.iter().for_each(|e| match e {
                    DiffEvent::ItemAdded(..) => stats.added += 1,
                    DiffEvent::ItemRemoved(..) => stats.removed += 1,
                    DiffEvent::ItemNoteChanged(..) => stats.note += 1,
                    DiffEvent::ItemStackSizeChanged(..) => stats.stack_size += 1,
                });

                dbg!(events);
            }
        });

    dbg!(&stats);
}
