# aws-pipeline

Goal: Build full data-processing pipeline on AWS utilising Lambdas, S3 and SQS.

## PS API

API: https://www.reddit.com/r/pathofexiledev/comments/48i4s1/information_on_the_new_stash_tab_api/d0kd1np/
> From then on, if you just wanted updates hourly, you'd store the current next change ID, wait an hour, make a request using the ID from an hour ago, and it would give you all the up-to-date information from changes made within that hour.

API2: https://www.reddit.com/r/pathofexiledev/comments/9ic7vq/are_the_stash_tab_api_responses_static/e6ox60h/
> Change ID 1044927-1025351-1035221-1190550-1024693 might contain Stash 1, 2, 3, 4, 5 with items 1.1, 1.2, 1.3, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1... 5.5.
>
>In an hour Stash 1 might change, and 1044927-1025351-1035221-1190550-1024693 will then respond with items 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 3.1... 5.5.
>
>This is so that if someone is traversing from the start of the stream, they don't load up old items. You'll also find a later Change ID containing the same update to Stash 1, where only items 1.4 and 1.5 is present.

In other words, each change id is associated with a fixed set of stash tabs that it refers to. 
When a change is brand new, the snapshots the change id refers to might be small in number.
Over a couple of seconds, a brand new change id is "filled" and associated with the latest stash tabs that changed.
When you request the same change id an hour later, the change id is still associated to the same stash tabs, but in order to avoid giving out stale data, the response data has changed and now contains the current (at the time of requesting) contents of each stash that the requested change id is associated with.

This means, that we can't just scrape every 10 seconds and expect to keep up. 
We actually need to scrap as fast as possible to not fall behind the stream/the current state of all stash tabs.
Because if we dont keep requesting change ids as fast as possible, we are falling behind on newer updates that might (and will) refer to other stashes than those that are associated with the next change id.

## TODO

- [ ] Terraform setup + GitHub Actions Pipeline to orchestrate infrastructure
- [ ] minimal Rust-based Lambda with local build & deploy setup
  - [ ] PS API Client with OAuth credentials
  - [ ] partial deserialization / look-ahead on next change id
  - [ ] s3 integration
  - [ ] sqs integration

## Architecture

![Arch](architecture.svg)

### Considerations S3 (old, under assumption to sample every 10s)
- object key schema: `<chunk-id>.zip`
- we will store a couple million zip files
- each zip file represents snapshots of all changed stash tabs compared to the last zip file
- we plan on sampling every 10s
<!-- - the average unzipped stash tab snapshot is estimated at around 11KB from previous sampling data
- however, each chunk usually contains 100-300 stash tabs (zipped size could be up to .5-1KB, lets estimate 5KB) -->
- at the fastest public sampling rate (roughly 1/s), I saw .8-1 GB/h in a PostgreSQL setup with indexes; lets assume we need the same with S3
- therefore the average unzipped S3 file size is estimated at around:
    <!-- - file_size = 11KB * 250 (conservative guess for # of stashes per chunk) = 2750 KB -->
    <!-- - 86400 seconds per day / 10s * 365 * file_size = 8.672.400.000.000 bytes = 8.67 TB -->
    - 365 days * 24 hours * 1 GB/h / 10 seconds per sampling = 876 GB per year (uncompressed)
    - 365 days * 24 hours * 3600 seconds per hour / 10 seconds = 3,153,600 files per year, 8640 files per day

### Requirements:
1. Python 3
2. diagrams & graphviz packages

### Questions
1. Q: Can I actually only scrape all 10s and still keep up with the stream?
   A: No, see above!


