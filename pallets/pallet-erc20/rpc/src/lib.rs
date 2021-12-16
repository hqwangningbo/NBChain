use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
pub use pallet_erc20_rpc_runtime_api::ERC20Api as ERC20RuntimeApi;
use pallet_erc20_rpc_runtime_api::ERC20Info;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[rpc]
pub trait ERC20Api<BlockHash, AccountId> {
    #[rpc(name = "get_erc20_info")]
    fn get_erc20_info(
        &self,
        owner: AccountId,
        at: Option<BlockHash>,
    ) -> Result<Option<ERC20Info<AccountId>>>;
}

/// A struct that implements the [`ERC20Api`].
pub struct ERC20<C, P> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<P>,
}

impl<C, P> ERC20<C, P> {
    /// Create new `ComingId` with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self { client, _marker: Default::default() }
    }
}

/// Error type of this RPC api.
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, Block, AccountId> ERC20Api<<Block as BlockT>::Hash, AccountId> for ERC20<C, Block>
    where
        Block: BlockT,
        AccountId: Codec,
        C: 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
        C::Api: ERC20RuntimeApi<Block, AccountId>,
{
    fn get_erc20_info(
        &self,
        owner: AccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Option<ERC20Info<AccountId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));
        api.get_erc20_info(&at, owner).map_err(|e| RpcError {
            code: ErrorCode::ServerError(Error::RuntimeError.into()),
            message: "Unable to get erc20 info.".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
