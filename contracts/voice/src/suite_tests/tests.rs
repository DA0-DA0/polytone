use cosmwasm_std::{Uint64, Addr};

use crate::suite_tests::suite::CREATOR_ADDR;

use super::suite::SuiteBuilder;

#[test]
fn test_update() {
    let mut suite = SuiteBuilder::default()
        .with_block_max_gas(Uint64::new(10))
        .build();

    suite.assert_block_max_gas(10);
    suite.assert_proxy_code(0);

    let proxy_code_new = suite.store_voice_contract();

    suite.update(
        Addr::unchecked(CREATOR_ADDR),
        proxy_code_new,
        50,
    )
    .unwrap();

    // assert that both fields updated succesfully
    suite.assert_block_max_gas(50);
    suite.assert_proxy_code(proxy_code_new);
}

#[test]
fn test_query_block_max_gas() {
    let mut suite = SuiteBuilder::default()
        .build();

    suite.assert_block_max_gas(0);

    suite.update(
        Addr::unchecked(CREATOR_ADDR),
        suite.voice_code,
        50,
    )
    .unwrap();

    suite.assert_block_max_gas(50);
}

#[test]
fn test_query_proxy_code_id() {
    let mut suite = SuiteBuilder::default()
        .build();
    
    suite.assert_proxy_code(0);

    suite.update(
        Addr::unchecked(CREATOR_ADDR),
        1,
        0,
    )
    .unwrap();

    suite.assert_proxy_code(1);
}