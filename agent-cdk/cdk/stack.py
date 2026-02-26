"""
AgentCore Runtime CDK stack.

What this stack manages
-----------------------
1. IAM execution role — the existing role is IMPORTED (not recreated) so
   ARNs in the console stay stable.  We attach an inline policy that grants
   the marketplace permissions needed for Anthropic cross-region models.

The S3 upload and runtime update are handled by deploy.py (boto3) because
the CDK SDK shim bundled with custom resources does not yet know about the
bedrock-agentcore service.  Run deploy.py after every cdk deploy.

Assumptions
-----------
- Runtime ID : set via agentcoreRuntimeId context key or AGENTCORE_RUNTIME_ID env var
- Role ARN   : imported, not recreated     (agentcoreRoleArn context key)
- S3 bucket  : already exists, imported    (agentcoreBucket context key)
- Region     : us-east-1
- Network    : PUBLIC
- Entry point: main.handler
"""

import os

import aws_cdk as cdk
import aws_cdk.aws_iam as iam
from constructs import Construct


class AgentCoreStack(cdk.Stack):
    def __init__(self, scope: Construct, construct_id: str, **kwargs) -> None:
        super().__init__(scope, construct_id, **kwargs)

        # ── configuration (override via cdk.json context) ─────────────────

        runtime_id: str = self.node.try_get_context("agentcoreRuntimeId") or os.environ.get(
            "AGENTCORE_RUNTIME_ID", ""
        )
        role_arn: str = self.node.try_get_context("agentcoreRoleArn") or os.environ.get(
            "AGENTCORE_ROLE_ARN", ""
        )
        bucket_name: str = self.node.try_get_context("agentcoreBucket") or os.environ.get(
            "AGENTCORE_BUCKET", ""
        )
        if not all([runtime_id, role_arn, bucket_name]):
            raise ValueError(
                "Missing required config. Set agentcoreRuntimeId, agentcoreRoleArn, "
                "and agentcoreBucket in cdk.json context or via environment variables "
                "(AGENTCORE_RUNTIME_ID, AGENTCORE_ROLE_ARN, AGENTCORE_BUCKET)."
            )
        default_model: str = self.node.try_get_context("defaultModelId") or os.environ.get(
            "DEFAULT_MODEL_ID",
            "us.anthropic.claude-sonnet-4-5-20251001-v1:0",
        )

        # ── 1. IAM role — import existing, patch marketplace perms ────────

        execution_role = iam.Role.from_role_arn(
            self, "ExecutionRole", role_arn, mutable=True
        )

        # Anthropic marketplace models require these actions on the execution
        # role that AgentCore Runtime assumes when calling Bedrock.
        execution_role.add_to_principal_policy(
            iam.PolicyStatement(
                sid="AllowBedrockMarketplaceModels",
                effect=iam.Effect.ALLOW,
                actions=[
                    "aws-marketplace:ViewSubscriptions",
                    "aws-marketplace:Subscribe",
                    "aws-marketplace:Unsubscribe",
                    "bedrock:InvokeModel",
                    "bedrock:InvokeModelWithResponseStream",
                ],
                resources=["*"],
            )
        )

        # ── outputs (consumed by deploy.py) ───────────────────────────────

        cdk.CfnOutput(self, "RuntimeId", value=runtime_id)
        cdk.CfnOutput(self, "BucketName", value=bucket_name)
        cdk.CfnOutput(self, "RoleArn", value=role_arn)
        cdk.CfnOutput(self, "DefaultModel", value=default_model)
