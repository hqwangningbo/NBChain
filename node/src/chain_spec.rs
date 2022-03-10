pub use nbchain_runtime::{
    AccountId, AuraConfig, BalancesConfig, ERC20Config, EthereumChainIdConfig, EthereumConfig,
    EvmConfig, GenesisAccount, GenesisConfig, GrandpaConfig, Signature, SudoConfig,
    SystemConfig, WASM_BINARY,
};
use sc_service::{ChainType, Properties};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{sr25519, Pair, Public, H160, U256};
use sp_core::crypto::Ss58Codec;
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
pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
    (get_from_seed::<AuraId>(s), get_from_seed::<GrandpaId>(s))
}

pub fn mainnet_config() -> Result<ChainSpec, String> {
    let mut properties = Properties::new();
    properties.insert("tokenSymbol".into(), "NB".into());
    properties.insert("tokenDecimals".into(), 18.into());
    let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
    Ok(ChainSpec::from_genesis(
        // Name
        "NBchain",
        // ID
        "mainnet",
        ChainType::Development,
        move || {
            nb_genesis(
                wasm_binary,
                // Initial PoA authorities
                vec![
                    (
                        AuraId::from_ss58check("5Hpah3QtBQ5zrstzp5j2q7DtSPBReEjUdPKfSsW3cPfJdMZN").unwrap(),
                        GrandpaId::from_ss58check("5Hpah3QtBQ5zrstzp5j2q7DtSPBReEjUdPKfSsW3cPfJdMZN").unwrap()
                    )
                ],
                // Sudo account
                AccountId::from_ss58check("5TGHffg29XJo5qjxTz1i66GD56umc5o75rz3WebS426Pvqzh").unwrap(),
                // Pre-funded accounts
                vec![
                    AccountId::from_ss58check("5TGHffg29XJo5qjxTz1i66GD56umc5o75rz3WebS426Pvqzh").unwrap(),
                ],
                true,
                vec![],
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

// pub fn testnet_config() -> Result<ChainSpec, String> {
//     let mut properties = Properties::new();
//     properties.insert("tokenSymbol".into(), "NB".into());
//     properties.insert("tokenDecimals".into(), 18.into());
//     let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
//     Ok(ChainSpec::from_genesis(
//         // Name
//         "NBchain",
//         // ID
//         "testnet",
//         ChainType::Live,
//         move || {
//             nb_genesis(
//                 wasm_binary,
//                 // Initial PoA authorities
//                 vec![(
//                     AuraId::from_slice(&(hex_literal::hex!["aea48c27a7f703a7f8acedf15b43e8fcbad0b7846e5fe32a0b2b75cb81d75306"][..])),
//                     GrandpaId::from_slice(&hex_literal::hex!["aea48c27a7f703a7f8acedf15b43e8fcbad0b7846e5fe32a0b2b75cb81d75306"][..])
//                 )],
//                 // Sudo account
//                 AccountId32::from(hex_literal::hex!["aea48c27a7f703a7f8acedf15b43e8fcbad0b7846e5fe32a0b2b75cb81d75306"]),
//                 // Pre-funded accounts
//                 vec![
//                     AccountId32::from(hex_literal::hex!["aea48c27a7f703a7f8acedf15b43e8fcbad0b7846e5fe32a0b2b75cb81d75306"]),
//                 ],
//                 true,
//                 vec![],
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

// pub fn mainnet_config() -> Result<ChainSpec, String> {
//     ChainSpec::from_json_bytes(&include_bytes!("../res/nbchain-mainnet-raw.json")[..])
// }

pub fn testnet_config() -> Result<ChainSpec, String> {
    ChainSpec::from_json_bytes(&include_bytes!("../res/nbchain-testnet-raw.json")[..])
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
                ],
                true,
                vec![
                    // Alith
                    H160::from(hex_literal::hex!["f24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"]),
                    // Baltathar
                    H160::from(hex_literal::hex!["3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0"]),
                    // Charleth
                    H160::from(hex_literal::hex!["798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc"]),
                    // Dorothy
                    H160::from(hex_literal::hex!["773539d4Ac0e786233D90A233654ccEE26a613D9"]),
                    // Ethan
                    H160::from(hex_literal::hex!["Ff64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB"]),
                    // Faith
                    H160::from(hex_literal::hex!["C0F0f4ab324C46e55D02D0033343B4Be8A55532d"]),
                    // Goliath
                    H160::from(hex_literal::hex!["7BF369283338E12C90514468aa3868A551AB2929"]),
                    // Heath
                    H160::from(hex_literal::hex!["931f3600a299fd9B24cEfB3BfF79388D19804BeA"]),
                    // Ida
                    H160::from(hex_literal::hex!["C41C5F1123ECCd5ce233578B2e7ebd5693869d73"]),
                    // Judith
                    H160::from(hex_literal::hex!["2898FE7a42Be376C8BC7AF536A940F7Fd5aDd423"]),
                    // Alice
                    H160::from(hex_literal::hex!["d43593c715fdd31c61141abd04a99fd6822c8558"]),
                ],
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
                ],
                true,
                vec![],
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

/// Configure initial storage state for FRAME modules.
fn nb_genesis(
    wasm_binary: &[u8],
    initial_authorities: Vec<(AuraId, GrandpaId)>,
    root_key: AccountId,
    endowed_accounts: Vec<AccountId>,
    _enable_println: bool,
    addresses: Vec<H160>,
) -> GenesisConfig {
    GenesisConfig {
        system: SystemConfig {
            // Add Wasm runtime to storage.
            code: wasm_binary.to_vec(),
            changes_trie_config: Default::default(),
        },
        balances: BalancesConfig {
            // Configure endowed accounts with initial balance of 1 << 60.
            balances: endowed_accounts
                .iter()
                .cloned()
                .map(|k| (k, 2100_000_000_000_000_000_000_000_000))
                .collect(),
        },
        aura: AuraConfig {
            authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
        },
        grandpa: GrandpaConfig {
            authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
        },
        sudo: SudoConfig {
            // Assign network admin rights.
            key: root_key,
        },
        ethereum_chain_id: EthereumChainIdConfig { chain_id: 1209u64 },
        evm: EvmConfig {
            accounts: addresses
                .into_iter()
                .map(|addr| {
                    (
                        addr,
                        GenesisAccount {
                            balance: U256::from(1_000_000_000_000_000_000_000u128),
                            nonce: Default::default(),
                            code: Default::default(),
                            storage: Default::default(),
                        },
                    )
                })
                .collect(),
        },
        ethereum: EthereumConfig {},
        erc20: ERC20Config {
            name: String::from("NB Token").into_bytes(),
            symbol: String::from("NB").into_bytes(),
            decimal: 18,
            owner: get_account_id_from_seed::<sr25519::Public>("Alice"),
        },
    }
}
