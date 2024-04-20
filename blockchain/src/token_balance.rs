use entity::nativetokenbalance::Entity as UserNativeTokenBalance;
use entity::nativetokenbalance::Model as UserNativeTokenBalanceModel;
use entity::users::Entity as User;
use ethers::{
    abi::ethereum_types::Public,
    core::utils::raw_public_key_to_address,
    providers::Provider,
    types::{Address, U256},
};
use ethers_contract::Multicall;
use ethers_providers::Http;
use redis::AsyncCommands;
use sea_orm::QueryFilter;
use sea_orm::Update;
use sea_orm::{entity::*, query::*, sea_query::Expr, DbBackend};
use sea_orm::{DatabaseConnection, EntityTrait};
use secp256k1::hashes::{sha256, Hash};
use secp256k1::PublicKey;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
pub async fn fetch_eth_balance(
    dbClient: DatabaseConnection,
    mut redisClient: redis::aio::MultiplexedConnection,
) {
    // connect to the network
    let client = Provider::<Http>::try_from("https://eth.drpc.org").unwrap();
    let mut multicall = Multicall::new(client.clone(), None).await.unwrap();

    let users = User::find().all(&dbClient).await;
    if let Ok(users) = &users {
        for user in users {
            let public_key = PublicKey::from_str(&user.public_key)
                .unwrap()
                .serialize_uncompressed();
            let address = raw_public_key_to_address(&public_key[1..]);
            // redisClient.set(address.to_string(), value);
            multicall.add_get_eth_balance(address, false);
        }
    }
    let users = users.unwrap();

    //Update the Balance every 1 minute in a loop and in another thread
    tokio::task::spawn(async move {
        loop {
            println!("Finding Balance");
            let balances: Vec<U256> = multicall.call_array().await.unwrap();
            for i in 0..balances.len() {
                let user = &users[i];
                let balance = balances[i].as_u64() as i64;

                let balance_from_cache = redisClient
                    .get::<String, i64>(user.public_key.to_string())
                    .await;

                if let Ok(cache_balance) = balance_from_cache {
                    if balance == cache_balance {
  
                        continue;
                    } else {
                        //Update Cache And Database

                        //Don't Care on Failure
                        println!("Updating Cache");
                        let insert_into_cache = redisClient
                            .set::<String, i64, i64>(user.public_key.to_string(), balance)
                            .await;

                        if insert_into_cache.is_err() {
                            println!("Cache Updation Failed");
                        }
                        println!("Updating Db");
                        let db_update =   dbClient.query_one(Statement::from_sql_and_values(DbBackend::Postgres, r#"INSERT INTO NativeTokenBalance VALUES (DEFAULT,$1, $2,$3) ON CONFLICT(public_key,token_name) DO UPDATE SET token_balance = $4;"#,vec![user.public_key.to_string().into(),String::from("ETH").into(),balance.into(),balance.into()])).await;
                        if db_update.is_err() {
                            println!("Db Updation Failed");
                        }
                    }
                } else {
                    println!("Updating Cache");
                    let insert_into_cache = redisClient
                        .set::<String, String, String>(user.public_key.to_string(), balance.to_string())
                        .await;

                    if let Err(e)  = insert_into_cache{
                        panic!("Cache Updation Failed {:?}",e);
                    }
                    println!("Updating Db");
                    let db_update =   dbClient.query_one(Statement::from_sql_and_values(DbBackend::Postgres, r#"INSERT INTO NativeTokenBalance VALUES (DEFAULT,$1, $2,$3) ON CONFLICT(public_key,token_name) DO UPDATE SET token_balance = $4;"#,vec![user.public_key.to_string().into(),String::from("ETH").into(),balance.into(),balance.into()])).await;
                    if db_update.is_err() {
                        println!("Db Updation Failed");
                    }
                }
            }
            println!("Sleeping");
            sleep(Duration::from_secs(10)).await;
            println!("Awaken");
        }
    });
}

#[cfg(test)]
mod tests {
    use entity::users::Entity as User;
    use entity::usertokens::Entity as UserTokens;
    use ethers::{
        abi::ethereum_types::Public,
        core::utils::raw_public_key_to_address,
        providers::Provider,
        types::{Address, U256},
    };
    use ethers_contract::Multicall;
    use ethers_providers::Http;
    use sea_orm::{DatabaseConnection, EntityTrait};
    use secp256k1::rand::rngs::OsRng;
    use secp256k1::{PublicKey, Secp256k1, SecretKey};
    use std::str::FromStr;

    #[tokio::test]
    async fn test_balance() {
        let client = Provider::<Http>::try_from("https://eth.drpc.org").unwrap();
        let mut multicall = Multicall::new(client.clone(), None).await.unwrap();
        let secp = Secp256k1::new();
        let mut i = 0;
        while i < 1827 {
            let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
            let public_key = public_key.serialize_uncompressed();

            let address = raw_public_key_to_address(&public_key[1..]);
            multicall.add_get_eth_balance(address, false);
            i = i + 1;
        }

        let balances: Vec<U256> = multicall.call_array().await.unwrap();
        let my_array: [U256; 1000] = [U256::zero(); 1000];
        assert_eq!(balances, my_array.to_vec());
        println!("{:?}", balances);
    }
}
