# DynamoDB Registry Design

Future architecture for metrics and advanced querying of shared recipes/prompts.

## Current State (S3-only)

- Simple file storage in S3
- Works for basic publish/import
- No query capability beyond listing
- Polling required for updates
- No metrics/analytics

## Recommended Architecture: DynamoDB + S3 Hybrid

| Store | What | Why |
|-------|------|-----|
| **DynamoDB** | Metadata, versions, metrics | Fast queries, counters, streams |
| **S3** | Actual content (JSON blobs) | Cheap storage, large payloads |

## DynamoDB Schema

```
Table: agentcore-registry
PK: source_id (e.g., "alice-my-recipe")
SK: version (e.g., "1.2.0")

Attributes:
- item_type: "recipe" | "prompt"
- name, description, owner
- s3_key: pointer to full content
- content_hash
- created_at, updated_at
- download_count (atomic increment)
- rating_sum, rating_count (for avg)
- tags[]

GSIs:
- by-type-updated: item_type (PK), updated_at (SK) → "latest recipes"
- by-owner: owner (PK), updated_at (SK) → "my published items"
- by-downloads: item_type (PK), download_count (SK) → "popular recipes"
```

## What This Enables

1. **Metrics**: `UpdateItem` with `ADD download_count :1` on each import
2. **Leaderboards**: Query GSI for top downloaded/rated items
3. **Real-time notifications**: DynamoDB Streams → Lambda → SNS/WebSocket
4. **Fast browsing**: Query metadata without downloading all S3 files
5. **Version history**: Query all SKs for a given PK

## Cost Comparison (small team ~10 users)

- S3-only: ~$0.02/month
- DynamoDB: ~$1-5/month (on-demand)
- Worth it for the query/metrics capability

## Implementation Steps

1. Create CDK stack for DynamoDB table with GSIs
2. Update publish commands to write metadata to DynamoDB, content to S3
3. Update list commands to query DynamoDB instead of listing S3
4. Update import commands to increment download_count
5. Add ratings API (rate_item, get_ratings)
6. Add DynamoDB Streams + Lambda for real-time update notifications

## Example Queries

```python
# Latest 10 recipes
table.query(
    IndexName='by-type-updated',
    KeyConditionExpression='item_type = :type',
    ExpressionAttributeValues={':type': 'recipe'},
    ScanIndexForward=False,
    Limit=10
)

# Most downloaded prompts
table.query(
    IndexName='by-downloads',
    KeyConditionExpression='item_type = :type',
    ExpressionAttributeValues={':type': 'prompt'},
    ScanIndexForward=False,
    Limit=10
)

# All versions of a specific item
table.query(
    KeyConditionExpression='source_id = :sid',
    ExpressionAttributeValues={':sid': 'alice-my-recipe'},
    ScanIndexForward=False  # newest first
)

# Increment download count
table.update_item(
    Key={'source_id': 'alice-my-recipe', 'version': '1.2.0'},
    UpdateExpression='ADD download_count :inc',
    ExpressionAttributeValues={':inc': 1}
)
```
