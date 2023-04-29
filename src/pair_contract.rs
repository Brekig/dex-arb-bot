use ethers::contract::abigen;
use ethers::contract::EthEvent;
// use ethers::prelude::*;
use ethers::prelude::{Http, Middleware, Provider};
use ethers::types::{Address, U256};
use futures::StreamExt;
use futures::{future::FutureExt, pin_mut, select};
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::oneshot;

abigen!(
    SushiswapV2Factory,
    r#"[{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"token0","type":"address"},{"indexed":true,"internalType":"address","name":"token1","type":"address"},{"indexed":false,"internalType":"address","name":"pair","type":"address"},{"indexed":false,"internalType":"uint256","name":"","type":"uint256"}],"name":"PairCreated","type":"event"},{"inputs":[{"internalType":"uint256","name":"","type":"uint256"}],"name":"allPairs","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"allPairsLength","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"createPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"stateMutability":"nonpayable","type":"function"},{"inputs":[],"name":"feeTo","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"feeToSetter","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"},{"internalType":"address","name":"","type":"address"}],"name":"getPair","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"migrator","outputs":[{"internalType":"address","name":"","type":"address"}],"stateMutability":"view","type":"function"},{"inputs":[],"name":"pairCodeHash","outputs":[{"internalType":"bytes32","name":"","type":"bytes32"}],"stateMutability":"pure","type":"function"},{"inputs":[{"internalType":"address","name":"_feeTo","type":"address"}],"name":"setFeeTo","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"name":"setFeeToSetter","outputs":[],"stateMutability":"nonpayable","type":"function"},{"inputs":[{"internalType":"address","name":"_migrator","type":"address"}],"name":"setMigrator","outputs":[],"stateMutability":"nonpayable","type":"function"}]"#
);

abigen!(
    UniswapV2Factory,
    r#"[{"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"payable":false,"stateMutability":"nonpayable","type":"constructor"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"token0","type":"address"},{"indexed":true,"internalType":"address","name":"token1","type":"address"},{"indexed":false,"internalType":"address","name":"pair","type":"address"},{"indexed":false,"internalType":"uint256","name":"","type":"uint256"}],"name":"PairCreated","type":"event"},{"constant":true,"inputs":[{"internalType":"uint256","name":"","type":"uint256"}],"name":"allPairs","outputs":[{"internalType":"address","name":"","type":"address"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"allPairsLength","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"tokenA","type":"address"},{"internalType":"address","name":"tokenB","type":"address"}],"name":"createPair","outputs":[{"internalType":"address","name":"pair","type":"address"}],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":true,"inputs":[],"name":"feeTo","outputs":[{"internalType":"address","name":"","type":"address"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[],"name":"feeToSetter","outputs":[{"internalType":"address","name":"","type":"address"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":true,"inputs":[{"internalType":"address","name":"","type":"address"},{"internalType":"address","name":"","type":"address"}],"name":"getPair","outputs":[{"internalType":"address","name":"","type":"address"}],"payable":false,"stateMutability":"view","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"_feeTo","type":"address"}],"name":"setFeeTo","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"},{"constant":false,"inputs":[{"internalType":"address","name":"_feeToSetter","type":"address"}],"name":"setFeeToSetter","outputs":[],"payable":false,"stateMutability":"nonpayable","type":"function"}]"#
);

//struct was wrong, needs to be the same as in the abi
#[derive(Clone, Debug, EthEvent)]
pub struct Swap {
    pub sender: Address,
    // #[serde(rename = "amount0In")]
    // pub amount0_in: U256,
    pub amount0in: U256,
    pub amount1in: U256,
    pub amount0out: U256,
    pub amount1out: U256,
    pub to: Address,
}

//Contracts
abigen!(
    SushiswapV2Pair,
    r#"[{"inputs":[],"name":"getReserves","outputs":[{"internalType":"uint112","name":"_reserve0","type":"uint112"},{"internalType":"uint112","name":"_reserve1","type":"uint112"},{"internalType":"uint32","name":"_blockTimestampLast","type":"uint32"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"},{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"sender","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount0In","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1In","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount0Out","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1Out","type":"uint256"},{"indexed":true,"internalType":"address","name":"to","type":"address"}],"name":"Swap","type":"event"}]"#
);

abigen!(
    UniswapV2Pair,
    r#"[{"anonymous":false,"inputs":[{"indexed":true,"internalType":"address","name":"sender","type":"address"},{"indexed":false,"internalType":"uint256","name":"amount0In","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1In","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount0Out","type":"uint256"},{"indexed":false,"internalType":"uint256","name":"amount1Out","type":"uint256"},{"indexed":true,"internalType":"address","name":"to","type":"address"}],"name":"Swap","type":"event"},{"inputs":[],"name":"getReserves","outputs":[{"internalType":"uint112","name":"_reserve0","type":"uint112"},{"internalType":"uint112","name":"_reserve1","type":"uint112"},{"internalType":"uint32","name":"_blockTimestampLast","type":"uint32"}],"stateMutability":"view","type":"function"},{"inputs":[{"internalType":"address","name":"","type":"address"}],"name":"balanceOf","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"}]"#
);

pub async fn get_sushiswap_pair_address<M: Middleware>(
    factory_contract: &SushiswapV2Factory<Provider<Http>>,
    token_a: &str,
    token_b: &str,
) -> Result<Address, Box<dyn Error + Send + Sync>> {
    let pair_address: Address = factory_contract
        .get_pair(Address::from_str(token_a)?, Address::from_str(token_b)?)
        .call()
        .await?;

    Ok(pair_address)
}
pub async fn get_uniswap_pair_address<M: Middleware>(
    factory_contract: &UniswapV2Factory<Provider<Http>>,
    token_a: &str,
    token_b: &str,
) -> Result<Address, Box<dyn Error + Send + Sync>> {
    let pair_address: Address = factory_contract
        .get_pair(Address::from_str(token_a)?, Address::from_str(token_b)?)
        .call()
        .await?;

    Ok(pair_address)
}

pub async fn get_pair_contract<M: Middleware>(
    provider: Arc<M>,
    pair_address: Address,
) -> UniswapV2Pair<M> {
    UniswapV2Pair::new(pair_address, provider)
}

pub async fn get_reserves<M: Middleware + 'static>(
    pair_contract: &UniswapV2Pair<M>,
) -> Result<(U256, U256, u32), Box<dyn std::error::Error>> {
    let (reserve0, reserve1, block_timestamp_last) = pair_contract.get_reserves().call().await?;
    Ok((
        U256::from(reserve0),
        U256::from(reserve1),
        block_timestamp_last,
    ))
}

pub async fn listen_to_swaps(
    uniswap_pair_contract: UniswapV2Pair<Provider<Http>>,
    sushiswap_pair_contract: UniswapV2Pair<Provider<Http>>,
    mut shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create an event stream for the "Swap" event on Uniswap.
    let uniswap_event = uniswap_pair_contract.event::<Swap>();
    let mut uniswap_stream = uniswap_event.stream().await?;

    // Create an event stream for the "Swap" event on Sushiswap.
    let sushiswap_event = sushiswap_pair_contract.event::<Swap>();
    let mut sushiswap_stream = sushiswap_event.stream().await?;

    let mut shutdown_fut = shutdown_rx.fuse();

    // Listen for the "Swap" event.
    loop {
        let next_uniswap_event = uniswap_stream.next().fuse();
        let next_sushiswap_event = sushiswap_stream.next().fuse();
        pin_mut!(next_uniswap_event, next_sushiswap_event);
        println!("Waiting for swap event...");
        select! {
            uniswap_event = next_uniswap_event => {
                println!("Raw Uniswap event data: {:?}", uniswap_event);
                match uniswap_event {
                    Some(swap_result) => {
                        match swap_result {
                            Ok(swap) => {
                                println!("Uniswap Swap Event:");
                                println!("  Sender: {:?}", swap.sender);
                                println!("  Amount0In: {:?}", swap.amount0in);
                                println!("  Amount1In: {:?}", swap.amount1in);
                                println!("  Amount0Out: {:?}", swap.amount0out);
                                println!("  Amount1Out: {:?}", swap.amount1out);
                                println!("  To: {:?}", swap.to);

                            },
                            Err(error) => {
                                println!("Error decoding Uniswap swap event: {:?}", error);
                            },
                        }
                    },
                    None => break,
                }
            },
            sushiswap_event = next_sushiswap_event => {
                println!("Raw Sushiswap event data: {:?}", sushiswap_event);
                match sushiswap_event {
                    Some(swap_result) => {
                        match swap_result {
                            Ok(swap) => {
                                println!("Sushiswap Swap Event:");
                                println!("  Sender: {:?}", swap.sender);
                                println!("  Amount0In: {:?}", swap.amount0in);
                                println!("  Amount1In: {:?}", swap.amount1in);
                                println!("  Amount0Out: {:?}", swap.amount0out);
                                println!("  Amount1Out: {:?}", swap.amount1out);
                                println!("  To: {:?}", swap.to);
                            },
                            Err(error) => {
                                println!("Error decoding Sushiswap swap event: {:?}", error);
                            },
                        }
                    },
                    None => break,
                }
            },
            _ = &mut shutdown_fut => {
                println!("Received shutdown signal");
                break;
            },
        }
    }

    Ok(())
}
