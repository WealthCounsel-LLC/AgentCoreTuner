#!/usr/bin/env python3
"""
Deploy the AgentCore Runtime bundle.

Usage:
    python deploy.py [--profile PROFILE] [--region REGION]

What it does:
1. Uses `uv pip install --target` to vendor all deps for Python 3.13 / ARM64
2. Zips main.py + all vendored deps (flat layout, same as Lambda)
3. Uploads the zip to the runtime's S3 bucket
4. Calls bedrock-agentcore-control UpdateAgentRuntime

Run this after every `cdk deploy` (for IAM changes) or on its own when
you only changed agent code.
"""

import argparse
import io
import json
import pathlib
import shutil
import subprocess
import sys
import tempfile
import zipfile

import boto3

# ── paths ─────────────────────────────────────────────────────────────────────

HERE = pathlib.Path(__file__).parent
AGENT_DIR = HERE / "agent"
CDK_JSON = HERE / "cdk.json"

# ── defaults from cdk.json context ────────────────────────────────────────────

_ctx = json.loads(CDK_JSON.read_text()).get("context", {})

DEFAULT_RUNTIME_ID = _ctx.get("agentcoreRuntimeId", "")
DEFAULT_ROLE_ARN = _ctx.get("agentcoreRoleArn", "")
DEFAULT_BUCKET = _ctx.get("agentcoreBucket", "")
BUNDLE_KEY = "source/agent-bundle.zip"

# AgentCore Runtime is always Linux ARM64
PLATFORM = "aarch64-manylinux2014"
PYTHON_VERSION = "3.13"


# ── helpers ───────────────────────────────────────────────────────────────────


def install_deps(target: pathlib.Path) -> None:
    """
    Vendor all dependencies using uv for the target platform.
    uv is the tool used by the official bedrock-agentcore-starter-toolkit.
    """
    uv = shutil.which("uv")
    if not uv:
        raise RuntimeError("uv not found. Install with: pip install uv")

    print(f"  Installing deps via uv (Python {PYTHON_VERSION} / {PLATFORM}) ...")
    subprocess.run(
        [
            uv, "pip", "install",
            "--target", str(target),
            "--python-version", PYTHON_VERSION,
            "--python-platform", PLATFORM,
            "--only-binary", ":all:",
            "--upgrade",
            "-r", str(AGENT_DIR / "requirements.txt"),
        ],
        check=True,
    )


def build_zip(deps_dir: pathlib.Path) -> bytes:
    """
    Zip main.py + all vendored deps into a flat layout (same as Lambda).
    Exclude __pycache__ to avoid platform-specific bytecode conflicts.
    """
    buf = io.BytesIO()
    with zipfile.ZipFile(buf, "w", zipfile.ZIP_DEFLATED) as zf:
        zf.write(AGENT_DIR / "main.py", "main.py")
        zf.write(AGENT_DIR / "requirements.txt", "requirements.txt")

        for dep_file in sorted(deps_dir.rglob("*")):
            if not dep_file.is_file():
                continue
            if "__pycache__" in dep_file.parts:
                continue
            arcname = dep_file.relative_to(deps_dir)
            zf.write(dep_file, arcname)

    return buf.getvalue()


def upload_zip(s3_client, bucket: str, key: str, data: bytes) -> None:
    print(f"  Uploading {len(data) / 1_048_576:.1f} MB to s3://{bucket}/{key} ...")
    s3_client.put_object(Bucket=bucket, Key=key, Body=data)
    etag = s3_client.head_object(Bucket=bucket, Key=key)["ETag"].strip('"')
    print(f"  Uploaded. ETag: {etag}")


def update_runtime(client, runtime_id: str, role_arn: str, bucket: str, key: str) -> None:
    print(f"  Updating AgentCore Runtime {runtime_id!r} ...")
    client.update_agent_runtime(
        agentRuntimeId=runtime_id,
        agentRuntimeArtifact={
            "codeConfiguration": {
                "code": {
                    "s3": {
                        "bucket": bucket,
                        "prefix": key,
                    }
                },
                "runtime": "PYTHON_3_13",
                "entryPoint": ["main.py"],
            }
        },
        networkConfiguration={"networkMode": "PUBLIC"},
        roleArn=role_arn,
    )
    print("  Runtime update submitted. It will be live in ~1-2 minutes.")


# ── main ──────────────────────────────────────────────────────────────────────


def main() -> None:
    parser = argparse.ArgumentParser(description="Deploy AgentCore Runtime bundle")
    parser.add_argument("--profile", default="default")
    parser.add_argument("--region", default="us-east-1")
    parser.add_argument("--runtime-id", default=DEFAULT_RUNTIME_ID)
    parser.add_argument("--role-arn", default=DEFAULT_ROLE_ARN)
    parser.add_argument("--bucket", default=DEFAULT_BUCKET)
    parser.add_argument(
        "--upload-only",
        action="store_true",
        help="Build and upload the bundle to S3 without updating the runtime.",
    )
    args = parser.parse_args()

    with tempfile.TemporaryDirectory() as tmpdir:
        deps_dir = pathlib.Path(tmpdir) / "deps"
        deps_dir.mkdir()

        print("Building bundle ...")
        install_deps(deps_dir)
        data = build_zip(deps_dir)
        print(f"  Bundle size: {len(data) / 1_048_576:.1f} MB")

    session = boto3.Session(profile_name=args.profile, region_name=args.region)
    s3 = session.client("s3")

    upload_zip(s3, args.bucket, BUNDLE_KEY, data)

    if args.upload_only:
        print(f"\nUploaded to s3://{args.bucket}/{BUNDLE_KEY}. Skipping runtime update.")
    else:
        agentcore = session.client("bedrock-agentcore-control", region_name=args.region)
        update_runtime(agentcore, args.runtime_id, args.role_arn, args.bucket, BUNDLE_KEY)
        print("\nDone. Check CloudWatch logs after ~2 min if the first invoke fails.")


if __name__ == "__main__":
    main()
