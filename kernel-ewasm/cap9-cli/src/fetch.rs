/// Module for the fetch interface.

use web3::futures::Future;
use web3::contract::{Contract, Options};
use web3::types::{Address, U256, H256};
// use web3::types::TransactionReceipt;
use web3::Transport;
use rustc_hex::FromHex;
use rustc_hex::ToHex;
// use ethabi::Token::Uint;
use crate::conn;
use crate::conn::EthConn;
use crate::project::LocalProject;
use cap9_std::proc_table::cap::*;
use pwasm_abi;
use std::fs::File;
use cap9_std::proc_table::ProcPointer;
use cap9_core::*;
use cap9_core::Error;
use cap9_core::Read;

pub fn list_procedures<T: Transport>(conn: &EthConn<T>, local_project: &LocalProject) -> Vec<Procedure> {
    let status_file = match local_project.status_file() {
        Some(status_file) => status_file,
        None => panic!("Project not deployed"),
    };
    let kernel_address = status_file.kernel_address;
    let n_proc_address: U256 = U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
    // println!("EntryProcAddress: 0x{}", entry_proc_address.to_hex());
    let store_val = conn.web3.eth().storage(kernel_address, n_proc_address, None).wait();
    // println!("EntryProc: {:?}", store_val);
    let n_procs: U256 = U256::from_big_endian(store_val.unwrap().as_bytes());
    let mut procs = Vec::new();
    for i in 1..(n_procs.as_u64() + 1) {
        let storage_address = get_idx_proc_address(i);
        let proc_key_raw: H256 = conn.web3.eth().storage(kernel_address, storage_address, None).wait().expect("proc key raw");
        let proc_key_bytes: [u8; 32] = proc_key_raw.to_fixed_bytes();
        let mut proc_bytes = [0; 24];
        proc_bytes.copy_from_slice(&proc_key_bytes[8..]);
        let proc_pointer: ProcPointer = ProcPointer::from_key(proc_bytes);
        let address_raw: H256 = conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_addr_ptr()), None).wait().expect("proc key raw");
        let address = Address::from_slice(&address_raw[12..]);

        let caps = Caps {
            proc_call: parse_procs(conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_CALL),
            proc_register: parse_procs(conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_REGISTER),
            proc_delete: parse_procs(conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_DELETE),
            proc_entry: parse_procs(conn, kernel_address.clone(), proc_pointer.clone(), CAP_PROC_ENTRY),
            store_write: parse_procs(conn, kernel_address.clone(), proc_pointer.clone(), CAP_STORE_WRITE),
            log: parse_procs(conn, kernel_address.clone(), proc_pointer.clone(), CAP_LOG),
            acc_call: parse_procs(conn, kernel_address.clone(), proc_pointer.clone(), CAP_ACC_CALL),
        };

        let procedure = Procedure {
            key: proc_bytes,
            index: i.into(),
            address,
            caps,
        };
        procs.push(procedure);
    }
    procs
}

fn parse_procs<T: Transport>(conn: &EthConn<T>, kernel_address: Address, proc_pointer: ProcPointer, cap_type: u8) -> Vec<Capability> {
    let n_proc_call_caps = U256::from_big_endian(&conn.web3.eth().storage(kernel_address, U256::from_big_endian(&proc_pointer.get_cap_type_len_ptr(cap_type)), None).wait().expect("proc key raw").to_fixed_bytes());
    let mut proc_call_caps = Vec::new();
    for i in 0..(n_proc_call_caps.as_u64() as u8 ) {
        let mut proc_call_caps_reader = CapReader {
            conn: conn,
            kernel_address,
            proc_pointer: proc_pointer.clone(),
            cap_type: cap_type,
            cap_index: i,
            current_val: 0,
        };
        match cap_type {
            CAP_PROC_CALL => {
                let procedure = ProcedureCallCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureCall(procedure.unwrap()));
            },
            CAP_PROC_REGISTER => {
                let procedure = ProcedureRegisterCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureRegister(procedure.unwrap()));
            },
            CAP_PROC_DELETE => {
                let procedure = ProcedureDeleteCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureDelete(procedure.unwrap()));
            },
            CAP_PROC_ENTRY => {
                let procedure = ProcedureEntryCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::ProcedureEntry(procedure.unwrap()));
            },
            CAP_STORE_WRITE => {
                let procedure = StoreWriteCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::StoreWrite(procedure.unwrap()));
            },
            CAP_LOG => {
                let procedure = LogCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::Log(procedure.unwrap()));
            },
            CAP_ACC_CALL => {
                let procedure = AccountCallCap::deserialize(&mut proc_call_caps_reader);
                proc_call_caps.push(Capability::AccountCall(procedure.unwrap()));
            },
            _ => panic!("invalid cap type"),
        }
    }
    proc_call_caps
}

struct CapReader<'a, T> where T: Transport {
    conn: &'a EthConn<T>,
    kernel_address: Address,
    proc_pointer: ProcPointer,
    cap_type: u8,
    cap_index: u8,
    current_val: u8,
}

impl<'a, T: Transport> Read<pwasm_abi::types::U256> for CapReader<'a, T> {
    fn read(&mut self, buf: &mut [pwasm_abi::types::U256]) -> Result<(), Error> {
        let next_val_ptr = self.proc_pointer.get_cap_val_ptr(self.cap_type, self.cap_index, self.current_val);
        let next_val = self.conn.web3.eth().storage(self.kernel_address, U256::from_big_endian(&next_val_ptr), None).wait().expect("proc key raw");
        self.current_val += 1;
        buf[0] = pwasm_abi::types::U256::from_big_endian(&next_val.to_fixed_bytes());
        Ok(())
    }

    fn remaining(&self) -> usize {
        1_usize
    }
}
#[derive(Clone, Debug)]
pub struct Caps {
    pub proc_call: Vec<Capability>,
    pub proc_register: Vec<Capability>,
    pub proc_delete: Vec<Capability>,
    pub proc_entry: Vec<Capability>,
    pub store_write: Vec<Capability>,
    pub log: Vec<Capability>,
    pub acc_call: Vec<Capability>,
}

#[derive(Clone, Debug)]
pub struct Procedure {
    pub key: [u8; 24],
    pub index: U256,
    pub address: Address,
    pub caps: Caps,
}

fn get_idx_proc_address(i: u64) -> U256 {
    let idx: u8 = i as u8;
    U256::from_big_endian(&[0xff, 0xff, 0xff, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, idx, 0x00, 0x00, 0x00])
}
