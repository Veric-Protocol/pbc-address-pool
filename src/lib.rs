//! pbc-address-pool

#[macro_use]
extern crate pbc_contract_codegen;

use pbc_contract_common::address::Address;
use pbc_contract_common::context::ContractContext;
use pbc_contract_common::sorted_vec_map::SortedVecMap;
use read_write_state_derive::ReadWriteState;
use read_write_rpc_derive::ReadWriteRPC;
use create_type_spec_derive::CreateTypeSpec;

#[derive(ReadWriteRPC, CreateTypeSpec, ReadWriteState)]
pub struct Vault {
    available: bool,
}

#[derive(ReadWriteRPC, CreateTypeSpec, ReadWriteState)]
pub struct NewVault {
    chain: String,
    address: String,
}

#[state]
pub struct ContractState {
    owner: Address,
    vaults: SortedVecMap<Address, SortedVecMap<String, SortedVecMap<String, Vault>>>,
}

#[init]
fn initialize(
    ctx: ContractContext,
) -> ContractState {

    let vault_storage: SortedVecMap<Address, SortedVecMap<String, SortedVecMap<String, Vault>>> = SortedVecMap::new();
    let state = ContractState {
        owner: ctx.sender,
        vaults: vault_storage,
    };

    state
}

#[action(shortname = 0x01)]
pub fn add_vault(
    context: ContractContext,
    mut state: ContractState,
    new_vaults: Vec<NewVault>,
) -> ContractState {

    assert!(new_vaults.len() != 0, "EMPTY VAULT VEC");

    // Prepare the Vaults Vec containing all the passed in Vaults
    let mut new_chain_map : SortedVecMap<String, SortedVecMap<String, Vault>> = SortedVecMap::new();

    for new_vault in new_vaults {
        if new_chain_map.contains_key(&new_vault.chain) {
            new_chain_map.get_mut(&new_vault.chain).unwrap().insert(new_vault.address, Vault { available: (true) });
        } else {
            let mut new_vault_map : SortedVecMap<String, Vault> = SortedVecMap::new();
            new_vault_map.insert(new_vault.address, Vault { available: (true) });
            new_chain_map.insert(new_vault.chain, new_vault_map);
        }
    }

    // Copy the existing Vaults into the new Vaults Vec, if any
    if state.vaults.contains_key(&context.sender) {
        let old_user_map = state.vaults.get(&context.sender).unwrap();
        for old_chain_map in old_user_map.iter() {
            let old_chain = old_chain_map.0.to_string();
            let old_vault_map = old_chain_map.1;
            for old_vault in old_vault_map.iter() {
                if new_chain_map.contains_key(&old_chain.clone()) {
                    new_chain_map.get_mut(&old_chain).unwrap().insert(old_vault.0.to_string(), Vault { available: old_vault.1.available });
                } else {
                    let mut new_vault_map : SortedVecMap<String, Vault> = SortedVecMap::new();
                    new_vault_map.insert(old_vault.0.to_string(), Vault { available: old_vault.1.available });
                    new_chain_map.insert(old_chain.clone(), new_vault_map);
                }
            }
        }
    } 

    // Replace the existing Vec with the new one
    state.vaults.insert(context.sender, new_chain_map);

    state
}

#[action(shortname = 0x02)]
pub fn remove_vault(
    context: ContractContext,
    mut state: ContractState,
    vault_address: String,
    vault_chain: String,
) -> ContractState {

    assert!(state.vaults.contains_key(&context.sender), "USER NOT EXIST");
    let chains_map = state.vaults.get_mut(&context.sender).unwrap();

    assert!(chains_map.contains_key(&vault_chain), "VAULT NOT EXIST");
    let vaults_map = chains_map.get_mut(&vault_chain).unwrap();

    assert!(vaults_map.contains_key(&vault_address), "VAULT NOT EXIST");
    assert!(vaults_map.get(&vault_address).unwrap().available, "VAULT IN USE");

    vaults_map.remove(&vault_address);

    state
}

#[action(shortname = 0x03)]
pub fn use_vault(
    context: ContractContext,
    mut state: ContractState,
    vault_address: String,
    vault_chain: String,
) -> ContractState {

    assert!(state.vaults.contains_key(&context.sender), "USER NOT EXIST");
    let chains_map = state.vaults.get_mut(&context.sender).unwrap();

    assert!(chains_map.contains_key(&vault_chain), "VAULT NOT EXIST");
    let vaults_map = chains_map.get_mut(&vault_chain).unwrap();

    assert!(vaults_map.contains_key(&vault_address), "VAULT NOT EXIST");
    assert!(vaults_map.get(&vault_address).unwrap().available, "VAULT IN USE");

    vaults_map.get_mut(&vault_address).unwrap().available = false;

    state
}

/* 
#[action(shortname = 0x04)]
pub fn use_availble_vault(
    context: ContractContext,
    mut state: ContractState,
    vault_chain: String,
) -> ContractState {

    assert!(state.vaults.contains_key(&context.sender), "USER NOT EXIST");

    let chains_map = state.vaults.get_mut(&context.sender).unwrap();
    assert!(chains_map.contains_key(&vault_chain), "NO AVAILABLE VAULT IN THIS CHAIN");

    let vaults_map = chains_map.get_mut(&vault_chain).unwrap();

    let mut found = false;
    for vault in vaults_map.iter_mut() {
        if vault.1.available {
            found = true;
            vault.1.available = false;
            break;
        }
    }

    assert!(found, "NO AVAILABLE VAULT IN THIS CHAIN");
    state
}
*/

#[action(shortname = 0x05)]
pub fn release_vault(
    context: ContractContext,
    mut state: ContractState,
    vault_address: String,
    vault_chain: String,
) -> ContractState {

    assert!(state.vaults.contains_key(&context.sender), "USER NOT EXIST");
    let chains_map = state.vaults.get_mut(&context.sender).unwrap();

    assert!(chains_map.contains_key(&vault_chain), "VAULT NOT EXIST");
    let vaults_map = chains_map.get_mut(&vault_chain).unwrap();

    assert!(vaults_map.contains_key(&vault_address), "VAULT NOT EXIST");

    vaults_map.get_mut(&vault_address).unwrap().available = true;

    state
}

#[action(shortname = 0x06)]
pub fn reset_vault(
    context: ContractContext,
    mut state: ContractState,
    new_vaults: Vec<NewVault>,
) -> ContractState {

    assert!(state.vaults.contains_key(&context.sender), "USER NOT EXIST");

    let mut new_chain_map : SortedVecMap<String, SortedVecMap<String, Vault>> = SortedVecMap::new();

    for new_vault in new_vaults {
        if new_chain_map.contains_key(&new_vault.chain) {
            new_chain_map.get_mut(&new_vault.chain).unwrap().insert(new_vault.address, Vault { available: (true) });
        } else {
            let mut new_vault_map : SortedVecMap<String, Vault> = SortedVecMap::new();
            new_vault_map.insert(new_vault.address, Vault { available: (true) });
            new_chain_map.insert(new_vault.chain, new_vault_map);
        }
    }

    // Replace the existing Vec with the new one
    state.vaults.insert(context.sender, new_chain_map);

    state
}