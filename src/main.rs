pub mod constants;
pub mod pair_contract;

use constants::{
    SUSHISWAP_ABI, SUSHISWAP_FACTORY_ADDRESS, TOKEN_SHIBA_INU_ADDRESS, TOKEN_WETH_ADDRESS,
    UNISWAP_ABI, UNISWAP_FACTORY_ADDRESS,
};

// use ethers::contract::abigen;
use ethers::{
    prelude::*,
    types::{Address, Filter},
};
// use futures::try_join;
use ethers::abi::Abi;
use ethers::contract::{BaseContract, Contract};
use ethers::providers::Ws;
use pair_contract::{
    get_reserves, get_sushiswap_pair_address, get_sushiswap_pair_contract,
    get_uniswap_pair_address, get_uniswap_pair_contract, Pair, SushiswapV2Factory, SushiswapV2Pair,
    UniswapV2Factory,
};
use std::env;
use std::str::FromStr;
use std::sync::Arc;
//To stop the program gracefully
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::signal;
use tokio::sync::oneshot;

abigen!(
    SushiswapV2Router,
    r#"[{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"token0","type":"address"},{"indexed":true,"internalType":"address","name":"token1","type":"address"},{"indexed":false,"internalType":"address","name":"pair","type":"address"},{"indexed":false,"internalType":"uint256","name":"","type":"uint256"}],"name":"PairCreated","type":"event"},{"inputs":[{"internalType":"uint256","name":"","type":"uint256"}],"name":"allPairs","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"allPairsLength","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"createPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"feeTo","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"feeToSetter","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"},{"internalType":"address","name":"","type":"address"}],"name":"getPair","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"migrator","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"pairCodeHash","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"address","name":"_feeTo","type":"address"}],"name":"setFeeTo","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"name":"setFeeToSetter","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_migrator","type":"address"}],"name":"setMigrator","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#
);
abigen!(
    UniswapV2Router,
    r#"[{"inputs":[{"internalType":"address","name":"_factory","type":"address"},{"internalType":"address","name":"_WETH","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"inputs":[],"name":"WETH","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"},{"internalType":"uint256","name":"amountADesired","type":"uint256"},{"internalType":"uint256","name":"amountBDesired","type":"uint256"},{"internalType":"uint256","name":"amountAMin","type":"uint256"},{"internalType":"uint256","name":"amountBMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"addLiquidity","outputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"amountB","type":"uint256"},{"internalType":"uint256","name":"liquidity","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"amountTokenDesired","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"addLiquidityETH","outputs":[{"internalType":"uint256","name":"amountToken","type":"uint256"},{"internalType":"uint256","name":"amountETH","type":"uint256"},{"internalType":"uint256","name":"liquidity","type":"uint256"}],"stateMutability":"payable","type":"function"},{"inputs":[],"name":"factory","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"reserveIn","type":"uint256"},{"internalType":"uint256","name":"reserveOut","type":"uint256"}],"name":"getAmountIn","outputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"reserveIn","type":"uint256"},{"internalType":"uint256","name":"reserveOut","type":"uint256"}],"name":"getAmountOut","outputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"}],"name":"getAmountsIn","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"}],"name":"getAmountsOut","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"reserveA","type":"uint256"},{"internalType":"uint256","name":"reserveB","type":"uint256"}],"name":"quote","outputs":[{"internalType":"uint256","name":"amountB","type":"uint256"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountAMin","type":"uint256"},{"internalType":"uint256","name":"amountBMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"removeLiquidity","outputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"amountB","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"removeLiquidityETH","outputs":[{"internalType":"uint256","name":"amountToken","type":"uint256"},{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"removeLiquidityETHSupportingFeeOnTransferTokens","outputs":[{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"bool","name":"approveMax","type":"bool"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"removeLiquidityETHWithPermit","outputs":[{"internalType":"uint256","name":"amountToken","type":"uint256"},{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"token","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountTokenMin","type":"uint256"},{"internalType":"uint256","name":"amountETHMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"bool","name":"approveMax","type":"bool"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"removeLiquidityETHWithPermitSupportingFeeOnTransferTokens","outputs":[{"internalType":"uint256","name":"amountETH","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"},{"internalType":"uint256","name":"liquidity","type":"uint256"},{"internalType":"uint256","name":"amountAMin","type":"uint256"},{"internalType":"uint256","name":"amountBMin","type":"uint256"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"},{"internalType":"bool","name":"approveMax","type":"bool"},{"internalType":"uint8","name":"v","type":"uint8"},{"internalType":"bytes32","name":"r","type":"bytes32"},{"internalType":"bytes32","name":"s","type":"bytes32"}],"name":"removeLiquidityWithPermit","outputs":[{"internalType":"uint256","name":"amountA","type":"uint256"},{"internalType":"uint256","name":"amountB","type":"uint256"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapETHForExactTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactETHForTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactETHForTokensSupportingFeeOnTransferTokens","outputs":[],"stateMutability":"payable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForETH","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForETHSupportingFeeOnTransferTokens","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountIn","type":"uint256"},{"internalType":"uint256","name":"amountOutMin","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapExactTokensForTokensSupportingFeeOnTransferTokens","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"amountInMax","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapTokensForExactETH","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"uint256","name":"amountOut","type":"uint256"},{"internalType":"uint256","name":"amountInMax","type":"uint256"},{"internalType":"address[]","name":"path","type":"address[]"},{"internalType":"address","name":"to","type":"address"},{"internalType":"uint256","name":"deadline","type":"uint256"}],"name":"swapTokensForExactTokens","outputs":[{"internalType":"uint256[]","name":"amounts","type":"uint256[]"}],"stateMutability":"nonpayable","type":"function"},{"stateMutability":"payable","type":"receive"}]"#
);

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

// async fn listen_all_events_sushiswap(contract: &SushiswapV2Pair<Provider<Ws>>) -> Result<()> {
//     let events = contract.events().from_block(16232696);
//     let mut stream = events.stream().await?.take(1);

//     while let Some(Ok(evt)) = stream.next().await {
//         match evt {
//             IERC20Events::ApprovalFilter(f) => println!("{f:?}"),
//             IERC20Events::TransferFilter(f) => println!("{f:?}"),
//         }
//     }

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let wss_url = &env::var("ETH_MAINNET_WS").expect("ETH_MAINNET_WS must be set");
    print!("Connecting to {}", wss_url);
    // Create a connection to the Ethereum network.
    let provider = Provider::<Ws>::connect(wss_url).await?;

    let client = Arc::new(provider);

    // Initialize the Sushiswap factory contract.
    let sushi_swap_factory_contract = SushiswapV2Factory::new(
        Address::from_str(SUSHISWAP_FACTORY_ADDRESS)?,
        client.clone(),
    );

    let sushiswap_pair_address: Address = get_sushiswap_pair_address::<Provider<Ws>>(
        &sushi_swap_factory_contract,
        TOKEN_SHIBA_INU_ADDRESS,
        TOKEN_WETH_ADDRESS,
    )
    .await
    .expect("failed getting sushiswap pair address");
    println!("Sushiswap Pair Address: {:?}", sushiswap_pair_address);

    // Initialize the Uniswap factory contract.
    let uniswapfactory_contract =
        UniswapV2Factory::new(Address::from_str(UNISWAP_FACTORY_ADDRESS)?, client.clone());

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
    let uniswap_pair_contract =
        get_uniswap_pair_contract(client.clone(), uniswap_pair_address).await;

    // Get the Sushiswap pair contract.
    let sushiswap_pair_contract =
        get_sushiswap_pair_contract(client.clone(), sushiswap_pair_address).await;

    // Now you can interact with both the Uniswap and Sushiswap pair contracts using their respective instances.
    // For example, you can get the reserves for the token pair on Uniswap:

    let (uniswap_reserve0, uniswap_reserve1, _) =
        get_reserves(&Pair::UniswapV2Pair(uniswap_pair_contract.clone())).await?;
    let (sushiswap_reserve0, sushiswap_reserve1, _) =
        get_reserves(&Pair::SushiswapV2Pair(sushiswap_pair_contract.clone())).await?;

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

    // let uniswap_abi: Abi = serde_json::from_str(SUSHISWAP_ABI).unwrap();

    // let uniswap_contract = SushiswapV2Router::new(uniswap_pair_contract.address(), uniswap_abi);

    // let sushiSwapABI: Abi = serde_json::from_str(UNISWAP_ABI).unwrap();

    // let sushiswap_contract = Contract::new(
    //     sushiswap_pair_contract.address(),
    //     sushiSwapABI,
    //     provider.clone(),
    // );

    // let uniswap_listener = listen_for_swaps(uniswap_contract);
    // let sushiswap_listener = listen_for_swaps(sushiswap_contract);

    // let ctrl_c_listener = signal::ctrl_c();
    // let listeners = tokio::select! {
    //     result = uniswap_listener => result,
    //     result = sushiswap_listener => result,
    //     _ = ctrl_c_listener => {
    //         println!("Ctrl+C pressed, shutting down...");
    //         Ok(())
    //     }
    // };

    // listeners?;

    //listen for swaps
    // Create a oneshot channel to signal shutdown
    // let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // // Spawn a task for listening to swaps
    // let listen_task = tokio::spawn(listen_to_swaps(
    //     uniswap_pair_contract,
    //     sushiswap_pair_contract,
    //     shutdown_rx,
    // ));
    // // Wait for the Ctrl+C signal
    // ctrl_c().await?;

    // // Send the shutdown signal
    // shutdown_tx
    //     .send(())
    //     .expect("Failed to send shutdown signal");

    // // Wait for the listen_to_swaps task to finish
    // match listen_task.await {
    //     Ok(_) => {
    //         println!("Exiting gracefully...");
    //     }
    //     Err(e) => {
    //         eprintln!("Listen task failed with error: {}", e);
    //         std::process::exit(1);
    //     }
    // }

    Ok(())
}
