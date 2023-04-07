use cosmwasm_std::{Empty, Uint64, Addr, WasmMsg, to_binary};
use cw_multi_test::{App, Executor, ContractWrapper, Contract, next_block};

use crate::msg::{InstantiateMsg, MigrateMsg};
use crate::msg::QueryMsg::{BlockMaxGas, ProxyCodeId};

const CREATOR_ADDR: &str = "creator";

fn voice_contract() -> Box<dyn Contract<Empty>> {
    let contract = ContractWrapper::new(
        crate::contract::execute,
        crate::contract::instantiate,
        crate::contract::query,
    )
    .with_migrate(crate::contract::migrate);
    Box::new(contract)
}

#[test]
fn test_update() {

    let mut app = App::default();
    let voice_id = app.store_code(voice_contract());

    let init_msg = InstantiateMsg {
        proxy_code_id: Uint64::from(voice_id),
        block_max_gas: Uint64::new(555),
    };

    let voice_address = app.instantiate_contract(
        voice_id, 
        Addr::unchecked(CREATOR_ADDR), 
        &init_msg, 
        &[], 
        "voice contract", 
        Some(CREATOR_ADDR.to_string()),
    )
    .unwrap();

    app.update_block(next_block);

    let new_voice_id = app.store_code(voice_contract());

    let migrate_msg = MigrateMsg::WithUpdate { 
        proxy_code_id: Uint64::from(new_voice_id),
        block_max_gas: Uint64::new(1),
    };

    app.execute(
        Addr::unchecked(CREATOR_ADDR), 
        WasmMsg::Migrate {
            contract_addr: voice_address.to_string(),
            new_code_id: new_voice_id,
            msg: to_binary(&migrate_msg).unwrap(),
        }
        .into(),
    )
    .unwrap();

    app.update_block(next_block);

    let block_max_gas: u64 = app
        .wrap()
        .query_wasm_smart(voice_address.clone(), &BlockMaxGas)
        .unwrap();

    let proxy_code_id: u64 = app
        .wrap()
        .query_wasm_smart(voice_address, &ProxyCodeId)
        .unwrap();

    assert_eq!(block_max_gas, 1);
    assert_eq!(proxy_code_id, new_voice_id);
}