# CLAUDE.md

## Project Overview

AWS AgentCore Manager — a Rust + Tauri v2 desktop app for managing and chatting with AWS Bedrock AgentCore Runtimes. Includes a Python AgentCore handler and AWS CDK infrastructure.

## Architecture

```
src/           Core Rust library (AWS CLI wrappers, caching, credentials, types)
  aws.rs       AWS CLI wrapper & response parsing
  cache.rs     Stale-while-revalidate local cache (5min agents, 24h models)
  types.rs     Shared data structures
ui-web/        Tauri v2 desktop application
  src/         Solid.js frontend (TypeScript)
  src-tauri/   Rust backend (Tauri commands, app state)
agent-cdk/     Python AWS infrastructure & agent handler
  agent/       Strands-based AgentCore Runtime handler (Python 3.13)
  cdk/         AWS CDK stack (IAM roles, marketplace model permissions)
  deploy.py    Bundles agent with uv, uploads to S3, updates runtime
tests/         Rust integration tests with JSON fixtures
  fixtures/    Mock AWS CLI responses for offline testing
```

## Tech Stack

- **Desktop App**: Rust + Tauri v2 + Solid.js
- **Agent Handler**: Python 3.13 + Strands + BedrockAgentCoreApp
- **Infrastructure**: AWS CDK v2, IAM, Bedrock, CloudWatch
- **AWS Region**: us-east-1

## Build / Test / Run

```bash
cargo build                         # Build desktop app
cargo test                          # Run unit tests (fixture-based, no AWS calls)
RUN_INTEGRATION_TESTS=1 cargo test  # Run integration tests (requires real AWS)
cargo run                           # Launch desktop app

# Agent deployment
python agent-cdk/deploy.py          # Bundle + deploy agent to AgentCore Runtime
cd agent-cdk && cdk deploy          # Deploy IAM infrastructure
```

## Git Workflow

- **Never commit directly to `main`.** Always create a feature branch for your changes.
- Branch naming: `claude/<issue-number>-<short-description>` (e.g., `claude/issue-42-add-dark-mode`)
- Create the branch from the latest `origin/main`: `git fetch origin && git checkout -b <branch> origin/main`
- Make commits on the feature branch, then push and open a PR back to `main`.
- This prevents merge conflicts with concurrent local development on `main`.

## Code Conventions

- Rust code follows standard Rust idioms (snake_case, Result-based error handling)
- AWS CLI responses are parsed from JSON using serde
- Tests use a FakeCli trait for dependency injection (no real AWS calls in unit tests)
- UI uses Tauri commands (`#[tauri::command]`) for Rust-frontend IPC
- Python agent uses uv for dependency management, not pip

## Key AWS Context

Set these environment variables (or override in `agent-cdk/cdk.json` context):
- `AGENTCORE_RUNTIME_ID`
- `AGENTCORE_ROLE_ARN`
- `AGENTCORE_BUCKET`

## Important Notes

- Agent bundle must exclude boto3/botocore (provided by runtime) to stay under size limit
- CDK imports existing IAM role rather than creating it (stable ARNs)
