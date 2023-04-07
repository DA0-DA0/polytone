use cosmwasm_std::{Uint64, Addr};

use crate::suite_tests::suite::CREATOR_ADDR;

use super::suite::SuiteBuilder;

#[test]
fn test_update() {
    let mut suite = SuiteBuilder::default()
        .with_block_max_gas(Uint64::new(10))
        .build();

    let block_max_pre = suite.query_block_max_gas();
    let proxy_code_pre = suite.query_proxy_code_id();

    assert_eq!(block_max_pre, 10);
    assert_eq!(proxy_code_pre, 0);

    let voice_code_new = suite.store_voice_contract();

    suite.update(
        Addr::unchecked(CREATOR_ADDR),
        voice_code_new,
        50,
    )
    .unwrap();
    
    let block_max_pre = suite.query_block_max_gas();
    let proxy_code_pre = suite.query_proxy_code_id();

    assert_ne!(block_max_pre, 0);
    assert_ne!(proxy_code_pre, 0);
}

#[test]
fn test_query_block_max_gas() {
    let mut suite = SuiteBuilder::default()
        .build();

    let block_max_gas = suite.query_block_max_gas();
    assert_eq!(block_max_gas, 0);

    suite.update(
        Addr::unchecked(CREATOR_ADDR),
        suite.voice_code,
        50,
    )
    .unwrap();

    let block_max_gas = suite.query_block_max_gas();
    assert_eq!(block_max_gas, 50);
}

#[test]
fn test_query_proxy_code_id() {
    let mut suite = SuiteBuilder::default()
        .build();
    
    let proxy_code_id = suite.query_proxy_code_id();
    assert_eq!(proxy_code_id, 0);

    suite.update(
        Addr::unchecked(CREATOR_ADDR),
        1,
        0,
    )
    .unwrap();

    let proxy_code_id = suite.query_proxy_code_id();
    assert_eq!(proxy_code_id, 1);
}