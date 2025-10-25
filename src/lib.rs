#![cfg_attr(not(feature = "std"), no_std)]
use concordium_std::*;

/// 32-byte identity commitment.
#[derive(Serial, Deserial, SchemaType, Clone, Copy, PartialEq, Eq)]
pub struct Commitment(pub [u8; 32]);

/// 32-byte nullifier to support one-time anonymous checks.
#[derive(Serial, Deserial, SchemaType, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nullifier(pub [u8; 32]);

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

/// Contract state for v9.
/// Must implement `Serial` (for `init`) and `DeserialWithState` (for `receive`).
#[derive(Serial, DeserialWithState)]
#[concordium(state_parameter = "StateApi")]
pub struct State {
    admin: AccountAddress,
    idps: StateSet<AccountAddress, StateApi>,
    verified: StateMap<AccountAddress, Commitment, StateApi>,
    used_nullifiers: StateSet<Nullifier, StateApi>,
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

#[derive(Serial, Deserial, SchemaType)]
pub struct AddIdpParam {
    pub idp: AccountAddress,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "add_idp",
    parameter = "AddIdpParam",
    error = "Error",
    mutable
)]
pub fn add_idp(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    ensure_admin(ctx, &host.state)?;
    let AddIdpParam { idp } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    host.state.idps.insert(idp);
    Ok(())
}

#[derive(Serial, Deserial, SchemaType)]
pub struct RemoveIdpParam {
    pub idp: AccountAddress,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "remove_idp",
    parameter = "RemoveIdpParam",
    error = "Error",
    mutable
)]
pub fn remove_idp(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    ensure_admin(ctx, &host.state)?;
    let RemoveIdpParam { idp } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;
    host.state.idps.remove(&idp);
    Ok(())
}

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
    mutable
)]
pub fn register(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let _idp_sender = ensure_idp(ctx, &host.state)?;
    let RegisterParam {
        subject,
        commitment,
    } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;

    // v9: no contains_key on StateMap; use get(..).is_none()
    ensure!(host.state.verified.get(&subject).is_none(), Error::AlreadyRegistered);

    host.state.verified.insert(subject, commitment);
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
    mutable
)]
pub fn revoke(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let _idp_sender = ensure_idp(ctx, &host.state)?;
    let RevokeParam { subject } = ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;

    ensure!(host.state.verified.get(&subject).is_some(), Error::NotRegistered);

    host.state.verified.remove(&subject);
    Ok(())
}

#[derive(Serial, Deserial, SchemaType)]
pub struct UseNullifierParam {
    pub nullifier: Nullifier,
}

#[receive(
    contract = "zk_kyc_registry",
    name = "use_nullifier",
    parameter = "UseNullifierParam",
    error = "Error",
    mutable
)]
pub fn use_nullifier(ctx: &ReceiveContext, host: &mut Host<State>) -> ContractResult<()> {
    let UseNullifierParam { nullifier } =
        ctx.parameter_cursor().get().map_err(|_| Error::Parse)?;

    ensure!(
        !host.state.used_nullifiers.contains(&nullifier),
        Error::NullifierUsed
    );

    host.state.used_nullifiers.insert(nullifier);

    // Optional: link the nullifier to the sender if they are verified (not emitted in v9).
    let _maybe_subject = match ctx.sender() {
        Address::Account(a) if host.state.verified.get(&a).is_some() => Some(a),
        _ => None,
    };

    Ok(())
}

#[receive(
    contract = "zk_kyc_registry",
    name = "is_verified",
    return_value = "bool"
)]
pub fn is_verified(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<bool> {
    let addr: AccountAddress = ctx.parameter_cursor().get()?;
    Ok(host.state.verified.get(&addr).is_some())
}

#[receive(
    contract = "zk_kyc_registry",
    name = "nullifier_used",
    return_value = "bool"
)]
pub fn nullifier_used(ctx: &ReceiveContext, host: &Host<State>) -> ReceiveResult<bool> {
    let n: Nullifier = ctx.parameter_cursor().get()?;
    Ok(host.state.used_nullifiers.contains(&n))
}
