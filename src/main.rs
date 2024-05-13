use std::sync::Arc;
use ethers::prelude::*;
use ethers::contract::abigen;
use lazy_static::lazy_static;
use sea_orm::prelude::Decimal;

use crate::core::consts;
use crate::util::{LibError, LibResult};

lazy_static! {
    pub static ref RPCNODE: RpcNode = RpcNode::new();
}

abigen!(
    ERC20,
    "data/abi/erc20.json",
);

pub struct RpcNode {
    provider: Arc<SignerMiddleware<Provider<Http>, LocalWallet>>,
    erc20_contract: ERC20<SignerMiddleware<Provider<Http>, LocalWallet>>,
}

impl RpcNode {
    fn new() -> Self {
       let provider = Arc::new({
            let provider = Provider::<Http>::try_from(consts::RPC_NODE.as_str()).unwrap();
            let wallet = consts::EOA_KEY.parse::<LocalWallet>().unwrap().with_chain_id(consts::CHAIN_ID.clone());
            SignerMiddleware::new(provider, wallet)
        });
        let address = consts::CONTRACT_ADDR.parse::<Address>().unwrap();
        let erc20_contract = ERC20::new(address, provider.clone());
        Self {
            provider,
            erc20_contract,
        }
    }

    pub async fn transfer(&self, to_addr: &str, amount: Decimal) -> LibResult<String> {
        let address = to_addr.parse::<Address>().unwrap();
        let amount = (amount * (*consts::USDT_DECIMAL)).trunc();
        let value = U256::from_dec_str(&amount.to_string()).unwrap();

        let gas_price = self.provider.get_gas_price().await.unwrap();
        let new_gas_price = gas_price + gas_price.saturating_mul(15.into()) / 100;
        tracing::info!("gas_price: {:?}, new_gas_price: {:?}", gas_price, new_gas_price);
        let receipt: Option<TransactionReceipt> = self.erc20_contract.transfer(address, value)
            .gas_price(new_gas_price)
            .send().await?
            .await?;
        tracing::info!("transfer tx: {:?}", receipt);
        if receipt.is_none() {
            return Err(LibError::TxnFailed);
        }
        let txn_hash = format!("{:#x}", receipt.unwrap().transaction_hash);
        Ok(txn_hash)
    }

}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_transfer() {
        let to_addr = "0xd8fce34b4280414866615b2cd9534716084b1647";
        let amount = Decimal::new(2, 0);
        let rsp = RPCNODE.transfer(to_addr, amount).await;
        tracing::info!("rsp {:?}", rsp);
        println!("rsp {:?}", rsp);
    }
}
