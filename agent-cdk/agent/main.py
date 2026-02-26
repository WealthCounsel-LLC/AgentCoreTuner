"""
AgentCore Runtime handler — Strands-based conversational agent.

All dials are configurable from the invocation payload:

  {
    "input": "Hello!",
    "model":         "us.anthropic.claude-sonnet-4-5-20251001-v1:0",
    "system_prompt": "You are a pirate."
  }
"""

import os
import logging

from bedrock_agentcore import BedrockAgentCoreApp
from strands import Agent
from strands.models import BedrockModel

logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = BedrockAgentCoreApp()

DEFAULT_MODEL = os.environ.get(
    "DEFAULT_MODEL_ID",
    "us.anthropic.claude-sonnet-4-5-20251001-v1:0",
)

DEFAULT_SYSTEM_PROMPT = os.environ.get(
    "DEFAULT_SYSTEM_PROMPT",
    "You are a helpful assistant.",
)


@app.entrypoint
def invoke(payload: dict) -> str:
    user_input: str = payload.get("input", "")
    if not user_input:
        return "(no input provided)"

    model_id: str = payload.get("model") or DEFAULT_MODEL
    system_prompt: str = payload.get("system_prompt") or DEFAULT_SYSTEM_PROMPT

    logger.info("invoke | model=%s", model_id)

    agent = Agent(
        model=BedrockModel(model_id=model_id),
        system_prompt=system_prompt,
    )

    return str(agent(user_input))


if __name__ == "__main__":
    app.run()
