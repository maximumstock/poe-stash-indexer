terraform {
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 4.16"
    }
  }

  required_version = ">= 1.2.0"
}

provider "aws" {
  region = "us-west-2"
}

resource "aws_s3_bucket" "poe-stash-stream-raw" {
  bucket = "poe-stash-stream-raw"
}

resource "aws_s3_bucket" "poe-stash-stream-raw-prod" {
  bucket = "poe-stash-stream-raw-prod"
}

resource "aws_s3_bucket_lifecycle_configuration" "archive-raw-prod" {
  bucket = aws_s3_bucket.poe-stash-stream-raw-prod.id

  rule {
    id     = "archive"
    status = "Enabled"

    transition {
      days          = 30
      storage_class = "ONEZONE_IA"
    }
  }
}

resource "aws_s3_bucket" "poe-stash-stream-daily-prod" {
  bucket = "poe-stash-stream-daily-prod"
}

resource "aws_s3_bucket_lifecycle_configuration" "archive-daily-prod" {
  bucket = aws_s3_bucket.poe-stash-stream-daily-prod.id

  rule {
    id     = "archive"
    status = "Enabled"

    transition {
      days          = 30
      storage_class = "ONEZONE_IA"
    }
  }
}

resource "aws_iam_role" "glue" {
  name = "glue"
  assume_role_policy = jsonencode({
    "Version" : "2012-10-17",
    "Statement" : [
      {
        "Sid" : "",
        "Effect" : "Allow",
        "Principal" : {
          "Service" : [
            "glue.amazonaws.com"
          ]
        },
        "Action" : "sts:AssumeRole"
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "poe-stash-stream-glue-role-policy-attachment" {
  role       = aws_iam_role.glue.name
  policy_arn = aws_iam_policy.glue-access.arn
}

resource "aws_iam_policy" "glue-access" {
  name        = "poe-stash-stream-glue-policy"
  description = "Models permissions for glue"
  policy      = data.aws_iam_policy_document.glue-policy.json
}

data "aws_iam_policy_document" "glue-policy" {
  statement {
    effect    = "Allow"
    actions   = ["cloudwatch:PutMetricData"]
    resources = ["*"]
  }
  statement {
    effect = "Allow"
    actions = [
      "logs:CreateLogGroup",
      "logs:CreateLogStream",
      "logs:PutLogEvents"
    ]
    resources = [
      "arn:aws:logs:*:*:/aws-glue/*",
      "arn:aws:logs:*:*:/customlogs/*"
    ]
  }
  statement {
    effect = "Allow"
    actions = [
      "s3:*",
    ]
    resources = ["*"]
  }
}

resource "aws_iam_role_policy_attachment" "poe-stash-stream-role-policy-attachment" {
  role       = aws_iam_role.glue.name
  policy_arn = aws_iam_policy.poe-stash-stream-policy.arn
}

resource "aws_iam_policy" "poe-stash-stream-policy" {
  name        = "poe-stash-stream-policy"
  description = "Models full access for stash data bucket"
  policy      = data.aws_iam_policy_document.poe-stash-stream-policy.json
}

data "aws_iam_policy_document" "poe-stash-stream-policy" {
  statement {
    effect = "Allow"
    actions = [
      "s3:PutObject",
      "s3:GetObject",
      "s3:ListBucket",
    ]
    resources = [
      "arn:aws:s3:::*/*",
      aws_s3_bucket.poe-stash-stream-raw.arn,
      aws_s3_bucket.poe-stash-stream-raw-prod.arn,
      aws_s3_bucket.poe-stash-stream-daily-prod.arn
    ]
  }
}

resource "aws_iam_user" "poe-stash-stream-user" {
  name = "poe-bucket-stream-user"
}

resource "aws_iam_user_policy" "poe-stash-stream-user-policy" {
  name   = "poe-stash-stream-user-policy"
  user   = aws_iam_user.poe-stash-stream-user.name
  policy = data.aws_iam_policy_document.poe-stash-stream-policy.json
}

resource "aws_iam_access_key" "poe-stash-stream-user-access-key" {
  user = aws_iam_user.poe-stash-stream-user.name
}

