# 1

In order to diff full stashes and generate statistical data (DiffStats) about events (DiffEvent), we need to:

## have a sql interface to the database
  - starting from a given change_id and league
  - we need to query all stash_records for that change_id and league
  - we need to group stash_records by account_name
    - we need a hashmap<item_id, { item & stash_id? }> with all the old items (starts empty)
    - we also need a second hashmap with all new items coming from this batch of stash_records within this change_id group
    - for each stash_record of a change_id batch:
      - we put it in the second, temporary hashmap
    - then we compare oldStash & newStash (see existing code)
    - this results in a vector of DiffEvents
    - we can annotate this vector like { events, timestamp of change_id, change_id, next_change_id, tick (incrementing counter that follows change_ids) }
