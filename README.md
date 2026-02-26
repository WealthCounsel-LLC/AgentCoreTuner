# AWS AgentCore Manager

A desktop application for managing and chatting with AWS Bedrock AgentCore Runtimes. Create agents, manage prompts and snapshots, invoke runtimes, and debug conversations — all from a native cross-platform app.

## Features

- **Agent Chat** — Stream conversations with AgentCore Runtimes in real time
- **Prompt Management** — Create, edit, and version system prompts with snapshot history
- **Runtime Management** — List, invoke, and monitor AgentCore Runtimes
- **Debug View** — Inspect full agent reasoning traces and tool-use events
- **S3 Sharing** — Export and import prompt snapshots via S3
- **Multi-Profile Auth** — Switch between AWS SSO profiles and accounts
- **Cross-Platform** — Runs on Linux, macOS (Apple Silicon & Intel), and Windows

## Tech Stack

| Layer | Technology |
|-------|-----------|
| Desktop Shell | [Tauri v2](https://v2.tauri.app/) (Rust) |
| Frontend | [SolidJS](https://www.solidjs.com/), TypeScript, [Tailwind CSS v4](https://tailwindcss.com/) |
| Agent Handler | Python 3.13, [Strands](https://strandsagents.com/), BedrockAgentCoreApp |
| Infrastructure | AWS CDK v2, IAM, Bedrock, CloudWatch |

## Prerequisites

- **Rust** 1.77.2+ (`rustup`)
- **Node.js** 18+ and npm
- **AWS CLI** v2, configured with SSO profiles
- **Linux only:** `libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf`

## Getting Started

```bash
# Install frontend dependencies
cd ui-web && npm install

# Run in development mode (hot-reload)
npm run tauri dev

# Build production installers (.deb, .AppImage, .dmg, .msi)
npm run tauri build
```

## Agent Deployment

The agent handler lives in `agent-cdk/` and runs on AWS AgentCore Runtimes.

```bash
# Bundle and deploy the agent to an existing runtime
python agent-cdk/deploy.py

# Deploy IAM infrastructure via CDK
cd agent-cdk && cdk deploy
```

## Project Structure

```
ui-web/                Tauri v2 desktop application
  src/                 SolidJS frontend (TypeScript, Tailwind)
  src-tauri/           Rust backend (Tauri commands, AWS SDK)
agent-cdk/             AWS infrastructure & agent handler
  agent/               Strands-based AgentCore Runtime handler
  cdk/                 CDK stack (IAM, Bedrock permissions)
  deploy.py            Bundle + deploy agent to S3 / runtime
```

## Releasing

A GitHub Actions workflow produces cross-platform installers on tag push:

```bash
git tag v0.1.0
git push origin v0.1.0
```

This creates a draft GitHub Release with `.deb`, `.AppImage`, `.dmg` (Apple Silicon + Intel), `.msi`, and `.nsis` installers. Review the draft and publish when ready.

## License

[MIT](LICENSE)
