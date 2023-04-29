pub mod constants;
pub mod pair_contract;

use constants::{
    SUSHISWAP_FACTORY_ADDRESS, TOKEN_SHIBA_INU_ADDRESS, TOKEN_WETH_ADDRESS, UNISWAP_FACTORY_ADDRESS,
};

use ethers::contract::abigen;
use ethers::{prelude::*, types::Address};
// use futures::try_join;
use pair_contract::{
    get_pair_contract, get_reserves, get_sushiswap_pair_address, get_uniswap_pair_address,
    listen_to_swaps, SushiswapV2Factory, UniswapV2Factory,
};
// use pair_contract::{SushiswapV2Pair, UniswapV2Pair};
use std::env;
use std::str::FromStr;
use std::sync::Arc;

//To stop the program gracefully
use tokio::signal::ctrl_c;
use tokio::sync::oneshot;

// factories begin
// abigen!(
//     SushiswapV2Factory,
//     r#"[{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"feeTo","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"feeToSetter","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"getPair","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"createPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_feeTo","type":"address"}],"name":"setFeeTo","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"name":"setFeeToSetter","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#
// );

// abigen!(
//     UniswapV2Factory,
//     r#"[{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"feeTo","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"feeToSetter","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"getPair","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"createPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_feeTo","type":"address"}],"name":"setFeeTo","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"name":"setFeeToSetter","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#
// );
//factories end

//contracts begin
// abigen!(
//     UniswapV2Pair,
//     r#"[{"inputs":[],"name":"getReserves","outputs":[{"internalType":"uint112","name":"_reserve0","type":"uint112"},{"internalType":"uint112","name":"_reserve1","type":"uint112"},{"internalType":"uint32","name":"_blockTimestampLast","type":"uint32"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"}]"#
// );

//contracts end

pub fn calculate_price_difference(
    reserve0_a: U256,
    reserve1_a: U256,
    reserve0_b: U256,
    reserve1_b: U256,
) -> f64 {
    let price_a = reserve1_a.to_string().parse::<f64>().unwrap()
        / reserve0_a.to_string().parse::<f64>().unwrap();
    let price_b = reserve1_b.to_string().parse::<f64>().unwrap()
        / reserve0_b.to_string().parse::<f64>().unwrap();
    let price_difference = ((price_a - price_b) / price_b) * 100.0;
    price_difference
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    // Create a connection to the Ethereum network.
    let provider = Provider::<Http>::try_from(&env::var("ETH_MAINNET").expect("missing env var"))?;

    let provider = Arc::new(provider);

    // Initialize the Sushiswap factory contract.
    let sushi_swap_factory_contract = SushiswapV2Factory::new(
        Address::from_str(SUSHISWAP_FACTORY_ADDRESS)?,
        provider.clone(),
    );

    // Get the pair address.
    // let sushiswap_pair_address: Address = sushi_swap_factory_contract
    //     .get_pair(
    //         Address::from_str(TOKEN_SHIBA_INU_ADDRESS)?,
    //         Address::from_str(TOKEN_WETH_ADDRESS)?,
    //     )
    //     .call()
    //     .await?;
    let sushiswap_pair_address: Address = get_sushiswap_pair_address::<Provider<Http>>(
        &sushi_swap_factory_contract,
        TOKEN_SHIBA_INU_ADDRESS,
        TOKEN_WETH_ADDRESS,
    )
    .await
    .expect("failed getting sushiswap pair address");
    println!("Sushiswap Pair Address: {:?}", sushiswap_pair_address);

    // Initialize the Uniswap factory contract.
    let uniswapfactory_contract = UniswapV2Factory::new(
        Address::from_str(UNISWAP_FACTORY_ADDRESS)?,
        provider.clone(),
    );

    // Get the pair address.
    let uniswap_pair_address: Address = get_uniswap_pair_address::<Provider<Http>>(
        &uniswapfactory_contract,
        TOKEN_SHIBA_INU_ADDRESS,
        TOKEN_WETH_ADDRESS,
    )
    .await
    .expect("failed getting uniswap pair address");

    println!("Uniswap Pair Address: {:?}", uniswap_pair_address);

    // Get the Uniswap pair contract.
    let uniswap_pair_contract = get_pair_contract(provider.clone(), uniswap_pair_address).await;

    // Get the Sushiswap pair contract.
    let sushiswap_pair_contract = get_pair_contract(provider.clone(), sushiswap_pair_address).await;

    // Now you can interact with both the Uniswap and Sushiswap pair contracts using their respective instances.
    // For example, you can get the reserves for the token pair on Uniswap:
    let uniswap_pair_contract_clone = uniswap_pair_contract.clone();
    let sushiswap_pair_contract_clone = sushiswap_pair_contract.clone();

    let (uniswap_reserve0, uniswap_reserve1, _) =
        get_reserves(&uniswap_pair_contract_clone).await?;
    let (sushiswap_reserve0, sushiswap_reserve1, _) =
        get_reserves(&sushiswap_pair_contract_clone).await?;

    println!("Uniswap Reserve0: {:?}", uniswap_reserve0);
    println!("Uniswap Reserve1: {:?}", uniswap_reserve1);

    println!("Sushiswap Reserve0: {:?}", sushiswap_reserve0);
    println!("Sushiswap Reserve1: {:?}", sushiswap_reserve1);

    let price_difference = calculate_price_difference(
        uniswap_reserve0,
        sushiswap_reserve0,
        uniswap_reserve1,
        sushiswap_reserve1,
    );
    println!("Price difference: {:.4}%", price_difference);

    //listen for swaps
    // Create a oneshot channel to signal shutdown
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // Spawn a task for listening to swaps
    let listen_task = tokio::spawn(listen_to_swaps(
        uniswap_pair_contract,
        sushiswap_pair_contract,
        shutdown_rx,
    ));
    // Wait for the Ctrl+C signal
    ctrl_c().await?;

    // Send the shutdown signal
    shutdown_tx
        .send(())
        .expect("Failed to send shutdown signal");

    // Wait for the listen_to_swaps task to finish
    match listen_task.await {
        Ok(_) => {
            println!("Exiting gracefully...");
        }
        Err(e) => {
            eprintln!("Listen task failed with error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
