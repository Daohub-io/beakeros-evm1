extern crate clap;
extern crate web3;
extern crate rustc_hex;
extern crate std;

use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
// use web3::types::TransactionReceipt;
use web3::Transport;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
// use ethabi::Token::Uint;
use crate::connection;
use crate::connection::EthConn;
use crate::project::*;
use cap9_std::proc_table::cap::*;
use pwasm_abi;
use std::fs::File;
use crate::deploy::web3::contract::tokens::Tokenize;
use crate::default_procedures::*;
use crate::utils::*;
use crate::constants::*;
use ethabi;
use failure::Error;

#[derive(Debug, Fail)]
pub enum ContractDeploymentError {
    #[fail(display = "failed to read interface into contract: {}", err)]
    ConstructionFailure {
        err: String,
    },
    #[fail(display = "incorrect parameters passed to constructor: {}", err)]
    BadParameters {
        err: String,
    },
    #[fail(display = "deployment failure: {}", err)]
    DeploymentFailure {
        err: String,
    },
}

// Deploy a contract
pub fn deploy_contract<T: Transport, P: Tokenize>(conn:  &EthConn<T>, code: Vec<u8>, interface: &[u8], params: P) -> Result<Contract<T>,ContractDeploymentError> {
    conn.web3.personal().unlock_account(conn.sender, "user", None).wait().unwrap();
    let code_hex: String = code.to_hex();
    Contract::deploy(conn.web3.eth(), interface)
        .map_err(|err| ContractDeploymentError::ConstructionFailure {err: format!("{:?}", err)})?
        .confirmations(REQ_CONFIRMATIONS)
        .options(Options::with(|opt| {
            opt.gas = Some(200_800_000.into())
        }))
        .execute( code_hex, params, conn.sender, )
        .map_err(|err| ContractDeploymentError::BadParameters {err: format!("{:?}", err)})?
        .wait()
        .map_err(|err| ContractDeploymentError::DeploymentFailure {err: format!("{:?}", err)})
}
