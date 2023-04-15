use cosmwasm_std::{
    testing::{mock_dependencies, mock_env, mock_info},
    to_binary, Uint64, WasmMsg,
};

use crate::{
    contract::{execute, instantiate},
    error::ContractError,
    msg::InstantiateMsg,
    state::CHANNEL,
};

#[test]
fn simple_note() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            pair: None,
            controller: None,
        },
    )
    .unwrap();
    CHANNEL
        .save(deps.as_mut().storage, &"some_channel".to_string())
        .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        crate::msg::ExecuteMsg::Execute {
            on_behalf_of: None,
            msgs: vec![WasmMsg::Execute {
                contract_addr: "some_addr".to_string(),
                msg: to_binary("some_msg").unwrap(),
                funds: vec![],
            }
            .into()],
            callback: None,
            timeout_seconds: Uint64::new(10000),
        },
    )
    .unwrap();
}

#[test]
fn controlled_note() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = mock_info("sender", &[]);
    let controller_info = mock_info("controller", &[]);

    instantiate(
        deps.as_mut(),
        env.clone(),
        info.clone(),
        InstantiateMsg {
            pair: None,
            controller: Some("controller".to_string()),
        },
    )
    .unwrap();
    CHANNEL
        .save(deps.as_mut().storage, &"some_channel".to_string())
        .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        controller_info.clone(),
        crate::msg::ExecuteMsg::Execute {
            on_behalf_of: Some("some_addr".to_string()),
            msgs: vec![WasmMsg::Execute {
                contract_addr: "some_addr".to_string(),
                msg: to_binary("some_msg").unwrap(),
                funds: vec![],
            }
            .into()],
            callback: None,
            timeout_seconds: Uint64::new(10000),
        },
    )
    .unwrap();

    // note is controlled but `on_behalf_of` is not set, should error
    let err = execute(
        deps.as_mut(),
        env.clone(),
        controller_info,
        crate::msg::ExecuteMsg::Execute {
            on_behalf_of: None,
            msgs: vec![WasmMsg::Execute {
                contract_addr: "some_addr".to_string(),
                msg: to_binary("some_msg").unwrap(),
                funds: vec![],
            }
            .into()],
            callback: None,
            timeout_seconds: Uint64::new(10000),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::OnBehalfOfNotSet);

    // note is controlled but not controller called execute, should error
    let err = execute(
        deps.as_mut(),
        env,
        info,
        crate::msg::ExecuteMsg::Execute {
            on_behalf_of: Some("some_addr".to_string()),
            msgs: vec![WasmMsg::Execute {
                contract_addr: "some_addr".to_string(),
                msg: to_binary("some_msg").unwrap(),
                funds: vec![],
            }
            .into()],
            callback: None,
            timeout_seconds: Uint64::new(10000),
        },
    )
    .unwrap_err();

    assert_eq!(err, ContractError::NotController)
}
