# Problem

I have the problem that I do not know how to analyse the data collected so far.
In this document I try to describe the current situation and the data format.

## Raw Data

We have collected player stash data in the form of `Stash`s.
Each stash record describes the contents of a single player stash at a given
point in time.

Our indexer regularly scrapes the official API in order to sample new stash records.
We assume an average scraping interval of 1s.
Each second, we scrape a batch of new stash records that describe a set of different stashes
and its new contents.

Each stash records describes only a single stash with a uniquely identifying id.
Therefore, moving an item between two stashes should generate two separate stash record updates.

A `Stash` looks roughly like this:

```rs

pub struct Stash {
    #[serde(rename(deserialize = "accountName"))]
    pub account_name: String,
    #[serde(rename(deserialize = "lastCharacterName"))]
    pub last_character_name: Option<String>,
    pub id: String,
    pub stash: Option<String>,
    #[serde(rename(deserialize = "stashType"))]
    pub stash_type: String,
    pub items: Vec<Item>,
    pub public: bool,
    pub league: Option<String>,
    pub created_at: NaiveDateTime,
    pub change_id: String,
    pub next_change_id: String,
}
```

The API does not offer any temporal metadata on these stash updates.
Therefore we manually add a monotonic timestamp and a monotonic chunk counter to indicate an ordering.

Here a chunk refers to the set of updates a specific stash records was delivered with.
The concept of a chunk has no deeper meaning (so far) and solely exists to describe how the API delivers data.

Each chunk might contain several (dozens/hundreds of) stash records.
A chunk might contain more than one stash record associated with a specific player account.

- set of chunks `C<sub>i</sub>`, with `C<sub>i</sub> = { R<sub>1</sub>, R<sub>2</sub>, ... R<sub>n</sub> }`

Since chunks heavily vary in size (0 - ~500 from what I've seen), we assume each chunk resembles player
activity within a fixed amount of time before it is published.

## Comparing Raw Stash Snapshots

The idea for this project is this:

- consume the chain of chunks and maintain a cache of what a player has in its whole inventory based on stash records at any given time/chunk
- for each chunk and each group of stash records of the same account:
  - if available, compare the previous contents of the updated stashes with the incoming updated state, and find the differences
  - update the stash cache

Finding the differences between two given snapshots of a given stash might result in observations like:

- an item was added
- an item was removed
- moving an item from stash A to stash B would result in two separate add & remove observations
  - we assume them to exist within the same chunk, based on our knowledge of when/how often PoE publishes these updates
- an existing item was changed in some way:
  - the `note` field changes, which contains eg. its selling price set by the owner
  - the `stack_size` field changes, because the item was a stackable item and parts were added or removed

# Questions:

1. How to track item age?

- Goal: Track item age NOT event age in relation to stash or a specific tab
- Track item age as number of stash updates since an item was added
- This means we only need one extra data point per item and one per stash:
  - the player stash to track # of stash updates as an increasing counter
  - a new item always has an age of equal to the current stash age
  - on remove, the age can be calculated for the event and the item is removed
  - on change, the age can be calculated for the event and the item's age is set to the current stash age

2. What event data to store and how?

- we have different events coming in different forms, eg. event age does not appear on every event type
- currently, we aggregate all events per player & 30-minute time window into a single .csv line for our dataset
- Problem: So far we had only simple counts, but event age data cannot be easily aggregated without losing information
  - Option 1: accept information loss (we don't even know if its that bad yet)
  - Option 2: store event data unaggregated
- Aggregation probably has to go at some point down the line anyways because it just loses so much granularity, therefore lets go with Option #2
  Processing of the dataset will then become a downstream problem/task, which sounds very reasonable
