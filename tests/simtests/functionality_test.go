package simtests

import (
	"encoding/base64"
	"encoding/json"
	"testing"

	w "github.com/CosmWasm/wasmvm/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/stretchr/testify/require"
)

// I can:
//
//   - Execute multiple messages (wasm and non-wasm) on a remote chain
//     and get a callback containing their response data.
//   - Execute multiple queries (wasm and non-wasm) on a remote chain
//     and get their responses in a callback.
func TestFunctionality(t *testing.T) {
	suite := NewSuite(t)

	path := suite.SetupPath(&suite.ChainA, &suite.ChainB)

	// Execute two messages, the first of which uses
	// polytone-tester to set some data in the transaction
	// response, and the second of which sets the proxy's staking
	// rewards receiver address to the voice address on the remote
	// chain.

	accountA := GenAccount(t, &suite.ChainA)
	dataMsg := `{"hello": { "data": "aGVsbG8K" }}`
	dataCosmosMsg := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Execute: &w.ExecuteMsg{
				ContractAddr: suite.ChainB.Tester.String(),
				Msg:          []byte(dataMsg),
				Funds:        []w.Coin{},
			},
		},
	}

	noDataCosmosMsg := w.CosmosMsg{
		Distribution: &w.DistributionMsg{
			SetWithdrawAddress: &w.SetWithdrawAddressMsg{
				Address: suite.ChainB.Voice.String(),
			},
		},
	}

	callback, err := suite.RoundtripExecute(t, path, &accountA, []any{dataCosmosMsg, noDataCosmosMsg})
	if err != nil {
		t.Fatal(err)
	}

	require.Equal(t, Callback{
		Success: []string{"aGVsbG8K", ""},
	}, callback)

	balanceQuery := w.QueryRequest{
		Bank: &w.BankQuery{
			Balance: &w.BalanceQuery{
				Address: suite.ChainB.Note.String(),
				Denom:   suite.ChainB.Chain.App.StakingKeeper.BondDenom(suite.ChainB.Chain.GetContext()),
			},
		},
	}

	history := QueryCallbackHistory(suite.ChainB.Chain, suite.ChainB.Tester)
	t.Log(history)
	testerQuery := TesterQuery{
		History: &Empty{},
	}
	queryBytes, err := json.Marshal(testerQuery)
	if err != nil {
		t.Fatal(err)
	}
	t.Log(string(queryBytes))

	historyQuery := w.QueryRequest{
		Wasm: &w.WasmQuery{
			Smart: &w.SmartQuery{
				ContractAddr: suite.ChainB.Tester.String(),
				Msg:          queryBytes,
			},
		},
	}

	callback, err = suite.RoundtripQuery(t, path, &accountA, []any{balanceQuery, historyQuery})
	if err != nil {
		t.Fatal(err)
	}

	require.Equal(t,
		Callback{
			Success: []string{
				base64.StdEncoding.EncodeToString([]byte(`{"amount":{"denom":"stake","amount":"100"}}`)), // contracts get made with 100 coins.
				base64.StdEncoding.EncodeToString([]byte(`{"history":[]}`))},
		}, callback)

}

// Generates two addresses from the same private key on chains B and
// C, then sends a message from each accounts proxy. The two addresses
// will have the same string representation, as the two chains have
// the same prefix, and the same local connection and channel ID. They
// also have the same remote port, as they are the first instantation
// of the same bytecode on chains with the same prefix.
//
// If these two different accounts get different addreses on chain A,
// it means that the contract is correctly distinguishing them based
// on some combination of local `(connection_id, channel_id)`, as
// those are the only parts of the messages that differ.
//
// Later tests will show that the contract does not change the address
// on chain A if a channel closes, which together means that the
// contract is correctly namespacing addresses based on connection_id.
func TestSameAddressDifferentChains(t *testing.T) {
	suite := NewSuite(t)

	pathCA := suite.SetupPath(&suite.ChainC, &suite.ChainA)
	pathBA := suite.SetupPath(&suite.ChainB, &suite.ChainA)

	friend := GenAccount(t, &suite.ChainB)

	// this follows the rules of Cosmos to induce the scenerio,
	// though signatures are not required for a message to be
	// sent from a malicious note contract, and anyone can
	// duplicate a chain, so you can imagine an attacker inducing
	// this scenerio at will.
	duplicate := friend.KeplrChainDropdownSelect(t, &suite.ChainC)

	require.Equal(t, friend.Address.String(), duplicate.Address.String())

	hello := `{"hello": { "data": "" }}`
	helloMsg := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Execute: &w.ExecuteMsg{
				ContractAddr: suite.ChainA.Tester.String(),
				Msg:          []byte(hello),
				Funds:        []w.Coin{},
			},
		},
	}

	c, err := suite.RoundtripExecute(t, pathCA, &duplicate, []any{helloMsg})
	if err != nil {
		t.Fatal(err)
	}
	b, err := suite.RoundtripExecute(t, pathBA, &friend, []any{helloMsg})
	if err != nil {
		t.Fatal(err)
	}
	require.Equal(t, Callback{Success: []string{""}}, c)
	require.Equal(t, c, b)

	history := QueryHelloHistory(suite.ChainA.Chain, suite.ChainA.Tester)
	require.Len(t, history, 2)
	require.NotEqual(t, history[0], history[1])
}

// TODO: Finished it once we have a way to retrieve proxy addr
func TestSuccessfulBankMsg(t *testing.T) {
	suite := NewSuite(t)

	path := suite.SetupPath(&suite.ChainA, &suite.ChainB)

	// Create an account on the other chain
	accountA := GenAccount(t, &suite.ChainA)
	hello := `{"hello": {"data": "aGVsbG8K"}}`
	helloMsg := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Execute: &w.ExecuteMsg{
				ContractAddr: suite.ChainB.Tester.String(),
				Msg:          []byte(hello),
				Funds:        []w.Coin{},
			},
		},
	}

	_, err := suite.RoundtripExecute(t, path, &accountA, []any{helloMsg})
	if err != nil {
		t.Fatal(err)
	}
	// Get account address
	history := QueryHelloHistory(suite.ChainB.Chain, suite.ChainB.Tester)
	proxyAddr := history[0]

	// Mint some tokens to the proxy address
	suite.ChainB.MintBondedDenom(t, sdk.AccAddress(proxyAddr))

	// Simple bank msg
	bankMsg := w.CosmosMsg{
		Bank: &w.BankMsg{
			Send: &w.SendMsg{
				ToAddress: suite.ChainB.Note.String(),
				Amount: []w.Coin{
					{Denom: suite.ChainB.Chain.App.StakingKeeper.BondDenom(suite.ChainB.Chain.GetContext()), Amount: "10000"},
				},
			},
		},
	}

	callback, err := suite.RoundtripExecute(t, path, &accountA, []any{bankMsg})
	if err != nil {
		t.Fatal(err)
	}
	t.Logf("callback = %v", callback)
}

// TODO: This function is erroring (Working)
// But we are waiting for the "TestSuccessfulBankMsg" test so we can be sure
// that the bank msgs are working as expected
func TestFailureMsg(t *testing.T) {
	suite := NewSuite(t)

	path := suite.SetupPath(&suite.ChainA, &suite.ChainB)

	accountA := GenAccount(t, &suite.ChainA)
	hello := `{"hello": {}}`
	helloMsg := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Execute: &w.ExecuteMsg{
				ContractAddr: suite.ChainA.Tester.String(),
				Msg:          []byte(hello),
				Funds:        []w.Coin{},
			},
		},
	}

	failedBankMsg := w.CosmosMsg{
		Bank: &w.BankMsg{
			Send: &w.SendMsg{
				ToAddress: suite.ChainB.Note.String(),
				Amount: []w.Coin{
					{Denom: "stake", Amount: "999999999"},
				},
			},
		},
	}

	// failed = {Success:[] Error:codespace: wasm, code: 5}
	// https://github.com/CosmWasm/wasmd/blob/7e936c7fffb6f489ed9ecb797a7a2823a032b10b/x/wasm/types/errors.go
	failed, _ := suite.RoundtripExecute(t, path, &accountA, []any{helloMsg, failedBankMsg})
	require.Equal(t, Callback{Error: "codespace: wasm, code: 5"}, failed)

	history := QueryHelloHistory(suite.ChainB.Chain, suite.ChainA.Tester)
	require.Len(t, history, 0)
}
