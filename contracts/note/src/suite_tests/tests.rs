use cosmwasm_std::{Addr, Uint64};

use super::suite::{SuiteBuilder, CREATOR_ADDR};


#[test]
fn test_update() {
    let mut suite = SuiteBuilder::default()
        .build();

    suite.assert_block_max_gas(0);

    suite.update(
        Addr::unchecked(CREATOR_ADDR),
        10,
    )
    .unwrap();

    suite.assert_block_max_gas(10);
}

#[test]
fn test_query_block_max_gas() {
    let suite = SuiteBuilder::default()
        .with_block_max_gas(Uint64::new(20))
        .build();

    suite.assert_block_max_gas(20);
}