# ZK KYC Registry Agent

Natural language interface for the ZK KYC Registry smart contract using LangChain.

## Installation
```bash
pip install -r requirements.txt
```

## Setup

Create `.env`:
```bash
OPENAI_API_KEY=your_key
CONCORDIUM_CONTRACT_INDEX=12265
CONCORDIUM_PASSWORD=your_wallet_password
```

## Usage
```bash
python agent.py
```

## Example Queries
```
You: Is 35Q7MXxjavBMvYhn6HU6NN1QL3uhzVHJXLoccuiCpcAvemzvZk verified?
🤖 Agent: ✅ Address 35Q7MXxjav... IS VERIFIED

You: Show me the commitment
🤖 Agent: Commitment found for 35Q7MXxjav...

You: Check if nullifier 88,88,88,... is used in domain 1
🤖 Agent: ✅ Nullifier HAS been used in domain 1

You: Who is the admin?
🤖 Agent: Admin address: 35Q7MXxjavBMvYhn6HU6NN1QL3uhzVHJXLoccuiCpcAvemzvZk
```

## Available Commands

- Check verification status
- Get commitments
- Use nullifiers (anonymous proofs)
- Check nullifier usage
- Register subjects (IDP only)
- Revoke subjects (IDP only)
- Get admin address
- Check IDP status