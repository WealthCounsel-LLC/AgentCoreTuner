#!/usr/bin/env python3
"""CDK app entry point for the AgentCore Runtime stack."""

import os

import aws_cdk as cdk
from stack import AgentCoreStack

app = cdk.App()

AgentCoreStack(
    app,
    "AgentCoreStack",
    env=cdk.Environment(
        account=os.environ["CDK_DEFAULT_ACCOUNT"],
        region=os.environ.get("CDK_DEFAULT_REGION", "us-east-1"),
    ),
)

app.synth()
