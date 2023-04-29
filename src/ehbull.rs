use ethers::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Result;
use std::convert::TryFrom;
use std::env;
use std::str::FromStr;
// use tokio::runtime::Runtime;
use web3::types::Address;
// use web3::Web3;
use web3::{ethabi::Bytes, types::U256};

pub const SUSHISWAP_V2_FACTORY_ABI: &str = r#"[{"inputs":[],"name":"allPairs","outputs":[{"internalType":"address[]","name":"","type":"address[]"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"_tokenA","type":"address"},{"internalType":"address","name":"_tokenB","type":"address"}],"name":"createPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"","type":"uint256"}],"name":"allPairs","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"allPairsLength","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"_tokenA","type":"address"},{"internalType":"address","name":"_tokenB","type":"address"}],"name":"getPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"stateMutability":"view","type":"function"}]"#;
const SUSHISWAP_V2_FACTORY_CONTRACT_ADDRESS: &str = "0xC0AEe478e3658e2610c5F7A4A2E1777cE9e4f2Ac";
const TOKEN_A_ADDRESS: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";
const TOKEN_B_ADDRESS: &str = "0x7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9";

#[derive(Serialize, Deserialize)]
struct Reserves {
    reserve0: U256,
    reserve1: U256,
}

async fn get_sushiswap_v2_pair_address() -> Result<Address> {
    dotenv::dotenv().ok();
    // Create an Ethereum provider with the default network (e.g., Mainnet)
    let provider = Provider::<Http>::try_from(
        "https://eth-mainnet.g.alchemy.com/v2/kSPplTdD5TsW5Xx7EUNwB8PEXRGrh-hf",
    )
    .expect("could not instantiate HTTP Provider");

    // Create a contract instance using the Sushiswap V2 Factory contract address and ABI
    let factory_contract = Contract::new(
        Address::from_str(SUSHISWAP_V2_FACTORY_CONTRACT_ADDRESS),
        SUSHISWAP_V2_FACTORY_ABI.parse()?,
        provider,
    );

    // Get the pair address using the getPair function
    let token_a = Address::from_str(TOKEN_A_ADDRESS)?;
    let token_b = Address::from_str(TOKEN_B_ADDRESS)?;
    let pair_address: Address = factory_contract
        .method::<_, Address>("getPair", (token_a, token_b), None)
        .await?;
    Ok(pair_address)
}

async fn fetch_reserves(
    web3: &web3::Web3<web3::transports::Http>,
    abi: &web3::ethabi::Contract,
    pair_address: &str,
) -> Result<Reserves> {
    let pair_addr = web3::types::Address::from_str(pair_address).unwrap();
    let call_output: Bytes = web3::contract::Contract::new(web3.eth(), pair_addr, abi.clone())
        .query(
            "getReserves",
            (),
            None,
            web3::contract::Options::default(),
            None,
        )
        .await
        .unwrap();

    let tokens: Vec<web3::ethabi::Token> = web3::ethabi::decode(
        &[web3::ethabi::ParamType::Tuple(vec![
            web3::ethabi::ParamType::Uint(256),
            web3::ethabi::ParamType::Uint(256),
            web3::ethabi::ParamType::Uint(32),
        ])],
        &call_output,
    )
    .unwrap();

    let reserves = match tokens.into_iter().next() {
        Some(web3::ethabi::Token::Tuple(values)) => {
            if let (web3::ethabi::Token::Uint(reserve0), web3::ethabi::Token::Uint(reserve1)) =
                (&values[0], &values[1])
            {
                Reserves {
                    reserve0: reserve0.clone(),
                    reserve1: reserve1.clone(),
                }
            } else {
                panic!("Unexpected values in tuple");
            }
        }
        _ => panic!("Unexpected token type"),
    };

    Ok(reserves)
}

#[tokio::main]
async fn main() -> Result<()> {
    let pair_address = get_sushiswap_v2_pair_address().await?;
    println!("Pair address: {}", pair_address);

    // let rt = Runtime::new().unwrap();
    // let web3 = web3::Web3::new(
    //     web3::transports::Http::new("https://mainnet.infura.io/v3/YOUR-INFURA-PROJECT-ID").unwrap(),
    // );

    // let uniswap_pair_abi = web3::ethabi::Contract::load(UNISWAP_PAIR_ABI.as_bytes()).unwrap();
    // let sushiswap_pair_abi = web3::ethabi::Contract::load(SUSHISWAP_PAIR_ABI.as_bytes()).unwrap();

    // rt.block_on(async {
    //     loop {
    //         let uniswap_reserves = fetch_reserves(&web3, &uniswap_pair_abi, UNISWAP_PAIR_ADDRESS)
    //             .await
    //             .unwrap();
    //         let sushiswap_reserves =
    //             fetch_reserves(&web3, &sushiswap_pair_abi, SUSHISWAP_PAIR_ADDRESS)
    //                 .await
    //                 .unwrap();

    //         let uniswap_price = uniswap_reserves
    //             .reserve1
    //             .to_string()
    //             .parse::<f64>()
    //             .unwrap()
    //             / uniswap_reserves
    //                 .reserve0
    //                 .to_string()
    //                 .parse::<f64>()
    //                 .unwrap();
    //         let sushiswap_price = sushiswap_reserves
    //             .reserve1
    //             .to_string()
    //             .parse::<f64>()
    //             .unwrap()
    //             / sushiswap_reserves
    //                 .reserve0
    //                 .to_string()
    //                 .parse::<f64>()
    //                 .unwrap();
    //         let price_difference = (uniswap_price - sushiswap_price) / sushiswap_price * 100.0;

    //         println!("Uniswap Price: {}", uniswap_price);
    //         println!("Sushiswap Price: {}", sushiswap_price);
    //         println!("Price Difference: {:.2}%", price_difference);

    //         // Wait for a while before fetching the data again
    //         tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
    //     }
    // });
    Ok(())
}
