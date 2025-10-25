#![cfg_attr(not(feature = "std"), no_std)]
use concordium_std::*;

/// 32-byte identity commitment.
#[derive(Serial, Deserial, SchemaType, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Commitment(pub [u8; 32]);

/// 32-byte nullifier to support one-time anonymous checks.
#[derive(Serial, Deserial, SchemaType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Nullifier(pub [u8; 32]);

/// Nullifier with domain separation for different use-cases
#[derive(Serial, Deserial, SchemaType, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct NullifierKey {
    pub domain: u16,         // e.g., 1 = KYC Check, 2 = Age Check, 3 = Residency
    pub nullifier: Nullifier,
}

#[derive(Reject, Serial, SchemaType, Debug, PartialEq, Eq)]
pub enum Error {
    Unauthorized,
    NotIdp,
    AlreadyRegistered,
    NotRegistered,
    NullifierUsed,
    Parse,
}
pub type ContractResult<T> = Result<T, Error>;

/// Events for indexer-friendly logging
#[derive(Serial, SchemaType, Debug)]
pub enum Event {
    IdpAdded { 
        idp: AccountAddress 
    },
    IdpRemoved { 
        idp: AccountAddress 
    },
    Registered { 
        idp: AccountAddress, 
        subject: AccountAddress, 
        commitment: Commitment,
        timestamp: Timestamp,
    },
    Revoked { 
        idp: AccountAddress, 
        subject: AccountAddress,
        timestamp: Timestamp,
    },
    NullifierUsed { 
        by: Option<AccountAddress>, 
        nullifier: Nullifier,
        domain: u16,
        timestamp: Timestamp,
    },
    AdminChanged {
        old_admin: AccountAddress,
        new_admin: AccountAddress,
        timestamp: Timestamp,
    },
}

/// Contract state
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "StateApi")]
pub struct State {
    admin: AccountAddress,
    idps: StateSet<AccountAddress, StateApi>,
    verified: StateMap<AccountAddress, Commitment, StateApi>,
    used_nullifiers: StateSet<NullifierKey, StateApi>,
    revoked_at: StateMap<AccountAddress, Timestamp, StateApi>, // Track revocation timestamps
}

impl State {
    fn new(sb: &mut StateBuilder, admin: AccountAddress, idps: Vec<AccountAddress>) -> Self {
        let mut idp_set = sb.new_set();
        for a in idps {
            idp_set.insert(a);
        }
        Self {
            admin,
            idps: idp_set,
            verified: sb.new_map(),
            used_nullifiers: sb.new_set(),
            revoked_at: sb.new_map(),
        }
    }
}

#[derive(Serial, Deserial, SchemaType)]
pub struct InitParams {
    pub admin: AccountAddress,
    pub idps: Vec<AccountAddress>,
}

#[init(contract = "zk_kyc_registry", parameter = "InitParams", error = "Error")]
pub fn init(ctx: &InitContext, sb: &mut StateBuilder) -> ContractResult<State> {
    let params: InitParams = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    Ok(State::new(sb, params.admin, params.idps))
}

fn ensure_admin(ctx: &ReceiveContext, state: &State) -> ContractResult<()> {
    ensure!(ctx.sender().matches_account(&state.admin), Error::Unauthorized);
    Ok(())
}

fn ensure_idp(ctx: &ReceiveContext, state: &State) -> ContractResult<AccountAddress> {
    let sender = match ctx.sender() {
        Address::Account(a) => a,
        _ => bail!(Error::NotIdp),
    };
    ensure!(state.idps.contains(&sender), Error::NotIdp);
    Ok(sender)
}

// ============================================================================
// ADMIN OPERATIONS
// ============================================================================

#[derive(Serial, Deserial, SchemaType)]
pub struct SetAdminParam {
    pub new_admin: AccountAddress,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "set_admin",
    parameter = "SetAdminParam",
    error = "Error",
    mutable,
    enable_logger
)]
pub fn set_admin(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure_admin(ctx, &host.state)?;
    let SetAdminParam { new_admin } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    
    let old_admin = host.state.admin;
    host.state.admin = new_admin;
    
    logger.log(&Event::AdminChanged {
        old_admin,
        new_admin,
        timestamp: ctx.metadata().slot_time(),
    }).ok();
    
    Ok(())
}

#[derive(Serial, Deserial, SchemaType)]
pub struct AddIdpParam {
    pub idp: AccountAddress,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "add_idp",
    parameter = "AddIdpParam",
    error = "Error",
    mutable,
    enable_logger
)]
pub fn add_idp(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure_admin(ctx, &host.state)?;
    let AddIdpParam { idp } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    
    host.state.idps.insert(idp);
    
    logger.log(&Event::IdpAdded { idp }).ok();
    
    Ok(())
}

#[derive(Serial, Deserial, SchemaType)]
pub struct AddIdpsBatchParam {
    pub idps: Vec<AccountAddress>,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "add_idps_batch",
    parameter = "AddIdpsBatchParam",
    error = "Error",
    mutable,
    enable_logger
)]
pub fn add_idps_batch(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    ensure_admin(ctx, &host.state)?;
    let AddIdpsBatchParam { idps } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    
    for idp in idps {
        host.state.idps.insert(idp);
        logger.log(&Event::IdpAdded { idp }).ok();
    }
    
    Ok(())
}

#[derive(Serial, Deserial, SchemaType)]
pub struct RemoveIdpParam {
    pub idp: AccountAddress,
}

#[derive(Serial, SchemaType)]
pub struct RemoveIdpResult {
    pub removed: bool,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "remove_idp",
    parameter = "RemoveIdpParam",
    error = "Error",
    mutable,
    enable_logger,
    return_value = "RemoveIdpResult"
)]
pub fn remove_idp(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<RemoveIdpResult> {
    ensure_admin(ctx, &host.state)?;
    let RemoveIdpParam { idp } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    
    let removed = host.state.idps.remove(&idp);
    
    if removed {
        logger.log(&Event::IdpRemoved { idp }).ok();
    }
    
    Ok(RemoveIdpResult { removed })
}

#[derive(Serial, Deserial, SchemaType)]
pub struct RemoveIdpsBatchParam {
    pub idps: Vec<AccountAddress>,
}

#[derive(Serial, SchemaType)]
pub struct RemoveIdpsBatchResult {
    pub removed_count: u32,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "remove_idps_batch",
    parameter = "RemoveIdpsBatchParam",
    error = "Error",
    mutable,
    enable_logger,
    return_value = "RemoveIdpsBatchResult"
)]
pub fn remove_idps_batch(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<RemoveIdpsBatchResult> {
    ensure_admin(ctx, &host.state)?;
    let RemoveIdpsBatchParam { idps } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    
    let mut removed_count = 0u32;
    for idp in idps {
        if host.state.idps.remove(&idp) {
            removed_count += 1;
            logger.log(&Event::IdpRemoved { idp }).ok();
        }
    }
    
    Ok(RemoveIdpsBatchResult { removed_count })
}

// ============================================================================
// IDP OPERATIONS
// ============================================================================

#[derive(Serial, Deserial, SchemaType)]
pub struct RegisterParam {
    pub subject: AccountAddress,
    pub commitment: Commitment,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "register",
    parameter = "RegisterParam",
    error = "Error",
    mutable,
    enable_logger
)]
pub fn register(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let idp = ensure_idp(ctx, &host.state)?;
    let RegisterParam {
        subject,
        commitment,
    } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;

    ensure!(host.state.verified.get(&subject).is_none(), Error::AlreadyRegistered);

    host.state.verified.insert(subject, commitment);
    
    // Clear revocation timestamp if re-registering
    host.state.revoked_at.remove(&subject);
    
    logger.log(&Event::Registered {
        idp,
        subject,
        commitment,
        timestamp: ctx.metadata().slot_time(),
    }).ok();
    
    Ok(())
}

#[derive(Serial, Deserial, SchemaType)]
pub struct RevokeParam {
    pub subject: AccountAddress,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "revoke",
    parameter = "RevokeParam",
    error = "Error",
    mutable,
    enable_logger
)]
pub fn revoke(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let idp = ensure_idp(ctx, &host.state)?;
    let RevokeParam { subject } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;

    ensure!(host.state.verified.get(&subject).is_some(), Error::NotRegistered);

    host.state.verified.remove(&subject);
    
    let timestamp = ctx.metadata().slot_time();
    host.state.revoked_at.insert(subject, timestamp);
    
    logger.log(&Event::Revoked {
        idp,
        subject,
        timestamp,
    }).ok();
    
    Ok(())
}

#[derive(Serial, Deserial, SchemaType)]
pub struct UseNullifierParam {
    pub nullifier: Nullifier,
    pub domain: u16, // Domain separation: 1 = KYC, 2 = Age, 3 = Residency, etc.
}

#[receive(
    contract = "zk_kyc_registry",
    name = "use_nullifier",
    parameter = "UseNullifierParam",
    error = "Error",
    mutable,
    enable_logger
)]
pub fn use_nullifier(
    ctx: &ReceiveContext,
    host: &mut Host<State>,
    logger: &mut Logger,
) -> ContractResult<()> {
    let UseNullifierParam { nullifier, domain } =
        ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;

    let key = NullifierKey { domain, nullifier };
    
    ensure!(
        !host.state.used_nullifiers.contains(&key),
        Error::NullifierUsed
    );

    host.state.used_nullifiers.insert(key);

    // Link nullifier to sender if they are verified
    let maybe_subject = match ctx.sender() {
        Address::Account(a) if host.state.verified.get(&a).is_some() => Some(a),
        _ => None,
    };

    logger.log(&Event::NullifierUsed {
        by: maybe_subject,
        nullifier,
        domain,
        timestamp: ctx.metadata().slot_time(),
    }).ok();

    Ok(())
}

// ============================================================================
// VIEW FUNCTIONS
// ============================================================================

#[receive(
    contract = "zk_kyc_registry",
    name = "is_verified",
    parameter = "AccountAddress",
    return_value = "bool"
)]
pub fn is_verified(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<bool> {
    let addr: AccountAddress = ctx.parameter_cursor().get()?;
    Ok(host.state.verified.get(&addr).is_some())
}

#[receive(
    contract = "zk_kyc_registry",
    name = "get_commitment",
    parameter = "AccountAddress",
    return_value = "Option<Commitment>"
)]
pub fn get_commitment(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Option<Commitment>> {
    let addr: AccountAddress = ctx.parameter_cursor().get()?;
    Ok(host.state.verified.get(&addr).map(|c| *c))
}

#[receive(
    contract = "zk_kyc_registry",
    name = "is_idp",
    parameter = "AccountAddress",
    return_value = "bool"
)]
pub fn is_idp(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<bool> {
    let addr: AccountAddress = ctx.parameter_cursor().get()?;
    Ok(host.state.idps.contains(&addr))
}

#[receive(
    contract = "zk_kyc_registry",
    name = "get_admin",
    return_value = "AccountAddress"
)]
pub fn get_admin(_ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<AccountAddress> {
    Ok(host.state.admin)
}

#[derive(Serial, Deserial, SchemaType)]
pub struct NullifierUsedParam {
    pub nullifier: Nullifier,
    pub domain: u16,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "nullifier_used",
    parameter = "NullifierUsedParam",
    return_value = "bool"
)]
pub fn nullifier_used(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<bool> {
    let NullifierUsedParam { nullifier, domain } = ctx.parameter_cursor().get()?;
    let key = NullifierKey { domain, nullifier };
    Ok(host.state.used_nullifiers.contains(&key))
}

#[receive(
    contract = "zk_kyc_registry",
    name = "get_revoked_at",
    parameter = "AccountAddress",
    return_value = "Option<Timestamp>"
)]
pub fn get_revoked_at(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<Option<Timestamp>> {
    let addr: AccountAddress = ctx.parameter_cursor().get()?;
    Ok(host.state.revoked_at.get(&addr).map(|t| *t))
}

// ============================================================================
// TESTS
// ============================================================================

#[concordium_cfg_test]
mod tests {
    use super::*;
    use concordium_std::test_infrastructure::*;

    #[concordium_test]
    fn test_init() {
        let mut ctx = TestInitContext::empty();
        let admin = AccountAddress([0u8; 32]);
        let idp1 = AccountAddress([1u8; 32]);
        
        let params = InitParams {
            admin,
            idps: vec![idp1],
        };
        
        let param_bytes = to_bytes(&params);
        ctx.set_parameter(&param_bytes);
        
        let mut sb = TestStateBuilder::new();
        let result = init(&ctx, &mut sb);
        
        assert!(result.is_ok());
    }

    #[concordium_test]
    fn test_domain_separation() {
        // Test that same nullifier can be used in different domains
        let nullifier = Nullifier([42u8; 32]);
        let key1 = NullifierKey { domain: 1, nullifier };
        let key2 = NullifierKey { domain: 2, nullifier };
        
        assert_ne!(key1, key2);
    }
}