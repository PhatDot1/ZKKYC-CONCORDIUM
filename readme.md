# ZK KYC Registry - Privacy-Preserving Identity Verification on Concordium

A zero-knowledge identity registry smart contract enabling privacy-preserving KYC verification with cryptographic commitments, domain-separated nullifiers, and immutable audit trails.

## 🎯 **What It Does**

Allows Identity Providers (IDPs) to register verified users on-chain without revealing personal information, while enabling users to prove their verified status anonymously through zero-knowledge proofs.

## 🔑 **Key Features**

### **Core Functionality**
- ✅ **Cryptographic Commitments** - Store 32-byte identity commitments instead of personal data
- ✅ **Privacy-Preserving Verification** - Prove verified status without revealing identity
- ✅ **Domain-Separated Nullifiers** - One-time anonymous checks across multiple use cases (KYC, age verification, residency)
- ✅ **Multi-IDP Support** - Decentralized trust through multiple authorized identity providers

### **Advanced Features**
- 🔐 **Admin Rotation** - Transfer admin privileges securely
- 📊 **Event Logging** - Indexer-friendly events with timestamps for off-chain analytics
- ⏱️ **Revocation Tracking** - Timestamped revocation history
- 🔄 **Batch Operations** - Add/remove multiple IDPs efficiently
- 🔍 **Rich Query API** - 10+ view functions for easy integration

## 🏗️ **Architecture**
```
┌─────────────────────────────────────────────────────────────┐
│                     ZK KYC Registry                         │
│                   (Concordium Smart Contract)               │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  Admin Layer:                                               │
│  • set_admin() - Transfer admin rights                      │
│  • add_idp() / remove_idp() - Manage identity providers     │
│  • add_idps_batch() / remove_idps_batch() - Bulk operations │
│                                                              │
│  IDP Layer:                                                 │
│  • register(subject, commitment) - Verify users             │
│  • revoke(subject) - Revoke verification                    │
│                                                              │
│  User Layer:                                                │
│  • use_nullifier(nullifier, domain) - Anonymous proof      │
│                                                              │
│  Query Layer:                                               │
│  • is_verified() - Check verification status                │
│  • get_commitment() - Retrieve commitment                   │
│  • is_idp() - Check IDP status                             │
│  • get_admin() - Get current admin                          │
│  • nullifier_used() - Check nullifier consumption          │
│  • get_revoked_at() - Get revocation timestamp             │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## 🚀 **Deployed Contract**

- **Network:** Concordium Testnet
- **Module Reference:** `d54e9484aad62e87863311538d2d08e56322ff1edbf14c427d51b7f7216eb05f`
- **Contract Address:** `12265:0`
- **Admin:** `35Q7MXxjavBMvYhn6HU6NN1QL3uhzVHJXLoccuiCpcAvemzvZk`

## 📚 **Use Cases**

### **1. Decentralized KYC**
Financial platforms verify users without storing personal data on-chain.

### **2. Age Verification** (Domain 2)
Prove you're 18+ without revealing your age or identity.

### **3. Residency Proof** (Domain 3)
Prove jurisdiction for regulatory compliance anonymously.

### **4. Sybil Resistance**
Prevent multi-accounting with one-time nullifiers per domain.

### **5. Privacy-First DeFi**
Comply with regulations while preserving user privacy.

## 🔧 **Technical Specifications**

### **State Structure**
```rust
pub struct State {
    admin: AccountAddress,
    idps: StateSet<AccountAddress>,
    verified: StateMap<AccountAddress, Commitment>,
    used_nullifiers: StateSet<NullifierKey>,
    revoked_at: StateMap<AccountAddress, Timestamp>,
}
```

### **Domain Separation**
```rust
pub struct NullifierKey {
    pub domain: u16,         // 1 = KYC, 2 = Age, 3 = Residency
    pub nullifier: Nullifier // 32-byte unique identifier
}
```

### **Events**
```rust
pub enum Event {
    IdpAdded { idp: AccountAddress },
    IdpRemoved { idp: AccountAddress },
    Registered { idp, subject, commitment, timestamp },
    Revoked { idp, subject, timestamp },
    NullifierUsed { by, nullifier, domain, timestamp },
    AdminChanged { old_admin, new_admin, timestamp },
}
```

## 📖 **API Reference**

### **Admin Functions**
| Function | Description | Gas |
|----------|-------------|-----|
| `set_admin(new_admin)` | Transfer admin privileges | ~1,200 NRG |
| `add_idp(idp)` | Add identity provider | ~1,100 NRG |
| `remove_idp(idp)` | Remove identity provider | ~1,100 NRG |
| `add_idps_batch(idps[])` | Add multiple IDPs | ~1,100/IDP |
| `remove_idps_batch(idps[])` | Remove multiple IDPs | ~1,100/IDP |

### **IDP Functions**
| Function | Description | Gas |
|----------|-------------|-----|
| `register(subject, commitment)` | Register verified user | ~1,094 NRG |
| `revoke(subject)` | Revoke verification | ~900 NRG |

### **User Functions**
| Function | Description | Gas |
|----------|-------------|-----|
| `use_nullifier(nullifier, domain)` | Consume nullifier for anonymous proof | ~1,020 NRG |

### **View Functions** (Read-only, ~400 NRG each)
- `is_verified(subject)` → bool
- `get_commitment(subject)` → Option<Commitment>
- `is_idp(address)` → bool
- `get_admin()` → AccountAddress
- `nullifier_used(nullifier, domain)` → bool
- `get_revoked_at(subject)` → Option<Timestamp>

## 🧪 **Testing**

### **Setup**
```bash
# Build contract
cargo concordium build --out zk_kyc_registry.wasm.v1 --schema-out schema.bin

# Deploy module
concordium-client module deploy zk_kyc_registry.wasm.v1 \
  --sender my-ccd --energy 70000 \
  --grpc-ip grpc.testnet.concordium.com --grpc-port 20000 --secure

# Initialize contract
concordium-client contract init <MODULE_REF> \
  --contract zk_kyc_registry --sender my-ccd --energy 50000 \
  --parameter-json init.json \
  --grpc-ip grpc.testnet.concordium.com --grpc-port 20000 --secure
```

### **Example: Register User**
```json
// register.json
{
  "subject": "35Q7MXxjavBMvYhn6HU6NN1QL3uhzVHJXLoccuiCpcAvemzvZk",
  "commitment": [[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]]
}
```
```bash
concordium-client contract update 12265 --subindex 0 \
  --entrypoint register --sender my-ccd --energy 40000 \
  --parameter-json register.json \
  --grpc-ip grpc.testnet.concordium.com --grpc-port 20000 --secure
```

### **Example: Anonymous Proof**
```json
// use_nullifier.json
{
  "nullifier": [[99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99,99]],
  "domain": 1
}
```

## 🔒 **Security Considerations**

### **Commitment Security**
- Commitments are 32-byte cryptographic hashes
- Off-chain: `commitment = hash(identity_data + salt)`
- On-chain: Only commitment stored, original data never revealed

### **Nullifier Privacy**
- One-time use per domain prevents replay attacks
- Domain separation allows same nullifier across different contexts
- Optional linking to verified accounts for audit trails

### **Access Control**
- Admin: Single address with full control (rotatable)
- IDPs: Whitelisted addresses can register/revoke
- Users: Can use nullifiers if verified

### **Immutability**
- Events provide permanent audit trail
- Revocations are timestamped, not deleted
- State transitions are transparent via events

## 📊 **Gas Costs**

| Operation | Gas Cost | Notes |
|-----------|----------|-------|
| Register | ~1,094 NRG | First registration |
| Revoke | ~900 NRG | Includes timestamp |
| Use Nullifier | ~1,020 NRG | With event logging |
| Queries | ~400 NRG | All view functions |
| Add IDP | ~1,100 NRG | Single operation |

## 🌐 **Integration Examples**

### **Frontend (Web3)**
```typescript
import { detectConcordiumProvider, serializeTypeValue } from '@concordium/web-sdk';

// Check if user is verified
const isVerified = await contract.invoke('is_verified', subject);

// Use nullifier for anonymous action
const tx = await provider.sendTransaction(
  account,
  'Update',
  {
    amount: 0n,
    address: { index: 12265n, subindex: 0n },
    receiveName: 'zk_kyc_registry.use_nullifier',
    message: serializeTypeValue({ nullifier, domain: 1 }, schema)
  }
);
```

### **Backend (Indexer)**
```python
# Listen for events
for event in contract_events:
    if event.type == "Registered":
        store_registration(event.subject, event.commitment, event.timestamp)
    elif event.type == "Revoked":
        mark_revoked(event.subject, event.timestamp)
```

## 🎓 **How It Works**

### **Registration Flow**
1. User completes KYC with IDP off-chain
2. IDP generates commitment: `H(identity_data || salt)`
3. IDP calls `register(user_address, commitment)`
4. Contract stores commitment, emits `Registered` event
5. User can now prove verification status

### **Anonymous Proof Flow**
1. User generates nullifier: `H(commitment || domain || nonce)`
2. User calls `use_nullifier(nullifier, domain)`
3. Contract checks: (a) nullifier unused, (b) user verified
4. Contract marks nullifier used, emits `NullifierUsed` event
5. Application grants access based on on-chain proof

## 🛣️ **Roadmap**

- [x] Core KYC registry
- [x] Domain-separated nullifiers
- [x] Event logging for indexers
- [x] Admin rotation
- [x] Batch IDP operations
- [x] Revocation timestamps
- [ ] Zero-knowledge proof integration (zk-SNARKs)
- [ ] Merkle tree for efficient verification
- [ ] Multi-signature admin
- [ ] Attribute-based verification (age, country, etc.)

## 📧 **Contact**

- GitHub: PhatDot1
- Twitter: ppw0_0x
- Email: patrickloughran121@outlook.com

---

**Built on Concordium Testnet** - Privacy-first blockchain with built-in identity layer