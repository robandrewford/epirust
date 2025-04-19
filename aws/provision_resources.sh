#!/bin/bash
# Example AWS provisioning script using AWS CLI

set -e

echo "Provisioning S3 bucket for epirust demo logs..."
aws s3api create-bucket --bucket epirust-demo-logs --region us-west-2

echo "Creating IAM role for epirust analysis tasks..."
aws iam create-role --role-name epirust-analysis-role --assume-role-policy-document file://trust-policy.json

echo "Done provisioning AWS resources."