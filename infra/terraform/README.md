# AWS Infrastructure

- S3 buckets for storing raw `.json.gzip` dumps every minute
- AWS Glue (script incoming) to aggregate and compact dumps to an hourly resolution
