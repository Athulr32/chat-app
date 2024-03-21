use ethers::providers::{Http, Middleware, Provider};

pub async fn fetch_eth_balance_for_given_address(address:String){

    let rpc_url = "https://rpc.sepolia.org";
    let provider = Provider::try_from(rpc_url).unwrap();

    let chain_id = provider.get_chainid().await.unwrap();
    let balance = provider.get_balance(address, None).await.unwrap();

    println!("{balance}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}