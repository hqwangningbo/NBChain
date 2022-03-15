pub use nbchain_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, Signature,
    SudoConfig, SystemConfig, WASM_BINARY, SessionConfig, opaque::SessionKeys,
};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public};
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};

// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
    where
        AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Generate an Aura authority key.
pub fn authority_keys_from_seed(s: &str) -> (AccountId, AuraId, GrandpaId) {
    (
        get_account_id_from_seed::<sr25519::Public>(s),
        get_from_seed::<AuraId>(s),
        get_from_seed::<GrandpaId>(s)
    )
}

// pub fn testnet_config() -> Result<ChainSpec, String> {
//     let mut properties = Properties::new();
//     properties.insert("tokenSymbol".into(), "NB".into());
//     properties.insert("tokenDecimals".into(), 18.into());
//     let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
//     Ok(ChainSpec::from_genesis(
//         // Name
//         "nbchain",
//         // ID
//         "testnet",
//         ChainType::Live,
//         move || {
//             nb_genesis(
//                 wasm_binary,
//                 // Initial PoA authorities
//                 vec![
//                     (
//                         AccountId::from_ss58check("5TGHffg29XJo5qjxTz1i66GD56umc5o75rz3WebS426Pvqzh").unwrap(),
//                         AuraId::from_ss58check("5HVywpNFg26mAzhDwh1bWX2iszmXHbFeRrh2VKs5qLXGrkVL").unwrap(),
//                         GrandpaId::from_ss58check("5GnQz21HZxCvK8gvaEX3VVAq3QP5JSoasRNo5HbpBePPu36Z").unwrap(),
//                     ),
//                     (
//                         AccountId::from_ss58check("5Tb1azWspGFtNHEdEoSQvJYahbD7B3o4eGEXLarncock4XFZ").unwrap(),
//                         AuraId::from_ss58check("5EvR75tnUoHR7dyLtwwrmXizqUewxn7jVeCvf8cheQTnYmqt").unwrap(),
//                         GrandpaId::from_ss58check("5GoTUw4EAi16ntdWtH69oXJrSH7puBa3G3PLKimirCLXy2V8").unwrap(),
//                     ),
//                 ],
//                 // Sudo account
//                 AccountId::from_ss58check("5TGHffg29XJo5qjxTz1i66GD56umc5o75rz3WebS426Pvqzh").unwrap(),
//                 // Pre-funded accounts
//                 vec![
//                     AccountId::from_ss58check("5TGHffg29XJo5qjxTz1i66GD56umc5o75rz3WebS426Pvqzh").unwrap()
//                 ],
//                 true,
//             )
//         },
//         // Bootnodes
//         vec![],
//         // Telemetry
//         None,
//         // Protocol ID
//         None,
//         // Properties
//         Some(properties),
//         // Extensions
//         None,
//     ))
// }

pub fn testnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/testnet-raw.json")[..])
}

pub fn development_config() -> Result<ChainSpec, String> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "NB".into());
    properties.insert("tokenDecimals".into(), 18.into());
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "Development",
        // ID
        "dev",
        ChainType::Development,
        move || {
            nb_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        Some(properties),
        // Extensions
        None,
    ))
}

pub fn local_testnet_config() -> Result<ChainSpec, String> {
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

    Ok(ChainSpec::from_genesis(
        // Name
        "Local Testnet",
        // ID
        "local_testnet",
        ChainType::Local,
        move || {
            nb_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![authority_keys_from_seed("Alice"), authority_keys_from_seed("Bob")],
                // Sudo account
                get_account_id_from_seed::<sr25519::Public>("Alice"),
                // Pre-funded accounts
                vec![
                    get_account_id_from_seed::<sr25519::Public>("Alice"),
                    get_account_id_from_seed::<sr25519::Public>("Bob"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie"),
                    get_account_id_from_seed::<sr25519::Public>("Dave"),
                    get_account_id_from_seed::<sr25519::Public>("Eve"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie"),
                    get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
                    get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
                ],
                true,
            )
        },
        // Bootnodes
        vec![],
        // Telemetry
        None,
        // Protocol ID
        None,
        // Properties
        None,
        // Extensions
        None,
    ))
}

fn nbchain_session_keys(aura: AuraId, grandpa: GrandpaId) -> SessionKeys {
    SessionKeys { aura, grandpa }
}

/// Configure initial storage state for FRAME modules.
fn nb_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AccountId, AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts.iter().cloned().map(|k| (k, 210_000_000_000_000_000_000_000)).collect(),
        },
        session: SessionConfig {
            keys: initial_authorities
                .iter()
                .map(|x| {
                    (
                        (x.0).clone(),
                        (x.0).clone(),
                        nbchain_session_keys(x.1.clone(), x.2.clone()),
                    )
                })
                .collect::<Vec<_>>(),
        },
        aura: Default::default(),
        grandpa: Default::default(),
        sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
    }
}

