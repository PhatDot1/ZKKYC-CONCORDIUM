#!/usr/bin/env python3
"""
ZK KYC Registry Agent - Natural Language Interface
"""

import os
import json
import re
import subprocess
import tempfile
from pathlib import Path
from typing import List

from dotenv import load_dotenv
from langchain.tools import tool
from langchain_openai import ChatOpenAI
from langchain.agents import create_react_agent, AgentExecutor
from langchain.prompts import PromptTemplate

load_dotenv()

OPENAI_API_KEY = os.getenv("OPENAI_API_KEY")
CONCORDIUM_NODE_HOST = os.getenv("CONCORDIUM_NODE_HOST", "grpc.testnet.concordium.com")
CONCORDIUM_NODE_PORT = os.getenv("CONCORDIUM_NODE_PORT", "20000")
CONTRACT_INDEX = os.getenv("CONCORDIUM_CONTRACT_INDEX", "12265")
CONTRACT_SUBINDEX = os.getenv("CONCORDIUM_CONTRACT_SUBINDEX", "0")

if not OPENAI_API_KEY:
    raise RuntimeError("OPENAI_API_KEY not set")

def strip_ansi(text: str) -> str:
    ansi_escape = re.compile(r'\x1b\[[0-9;]*m')
    return ansi_escape.sub('', text)

def run_concordium_cmd(cmd: List[str]) -> str:
    try:
        result = subprocess.run(cmd, text=True, check=True, capture_output=True)
        output = result.stdout + result.stderr
        return strip_ansi(output).strip() or "Command succeeded"
    except subprocess.CalledProcessError as e:
        return f"Error: {strip_ansi(e.stdout + e.stderr)}"

@tool
def check_verified(address: str) -> str:
    """Check if a Concordium address is verified."""
    fd, path = tempfile.mkstemp(suffix=".json")
    os.close(fd)
    temp_file = Path(path)
    
    try:
        with temp_file.open("w") as f:
            json.dump(address, f)
        
        cmd = [
            "concordium-client", "contract", "invoke",
            CONTRACT_INDEX, "--subindex", CONTRACT_SUBINDEX,
            "--entrypoint", "is_verified",
            "--parameter-json", str(temp_file),
            "--grpc-ip", CONCORDIUM_NODE_HOST,
            "--grpc-port", CONCORDIUM_NODE_PORT,
            "--secure"
        ]
        
        result = run_concordium_cmd(cmd)
        
        if "true" in result.lower():
            return f"✅ Address {address[:12]}... IS VERIFIED"
        elif "false" in result.lower():
            return f"❌ Address {address[:12]}... is NOT verified"
        return result
    finally:
        temp_file.unlink(missing_ok=True)

@tool
def get_admin() -> str:
    """Get the current admin address."""
    cmd = [
        "concordium-client", "contract", "invoke",
        CONTRACT_INDEX, "--subindex", CONTRACT_SUBINDEX,
        "--entrypoint", "get_admin",
        "--grpc-ip", CONCORDIUM_NODE_HOST,
        "--grpc-port", CONCORDIUM_NODE_PORT,
        "--secure"
    ]
    return f"Admin: {run_concordium_cmd(cmd)}"

AGENT_PROMPT = """Answer questions about the ZK KYC Registry smart contract.

Tools: {tools}
Tool Names: {tool_names}

Format:
Question: {input}
Thought: {agent_scratchpad}
"""

def create_agent():
    tools = [check_verified, get_admin]
    llm = ChatOpenAI(model="gpt-4o-mini", temperature=0)
    prompt = PromptTemplate.from_template(AGENT_PROMPT)
    agent = create_react_agent(llm, tools, prompt)
    return AgentExecutor(
        agent=agent, 
        tools=tools, 
        verbose=True, 
        max_iterations=3,
        handle_parsing_errors=True  # Added Bugfix
    )

def main():
    print("=" * 80)
    print("ZK KYC Registry Agent")
    print("=" * 80)
    print(f"Contract: {CONTRACT_INDEX}:{CONTRACT_SUBINDEX}")
    print("Type 'quit' to exit\n")
    
    agent = create_agent()
    
    while True:
        try:
            query = input("You: ").strip()
            if not query or query.lower() in ['quit', 'exit', 'q']:
                print("Goodbye! 👋")
                break
            
            result = agent.invoke({"input": query})
            print(f"\n🤖 {result['output']}\n")
        except KeyboardInterrupt:
            print("\n\nGoodbye! 👋")
            break
        except Exception as e:
            print(f"❌ Error: {e}\n")

if __name__ == "__main__":
    main()