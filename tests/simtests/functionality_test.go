package simtests

import (
	"encoding/json"
	"fmt"
	"testing"

	w "github.com/CosmWasm/wasmvm/types"
	"github.com/stretchr/testify/require"
)

type Tester string

const (
	testBinary Tester = "aGVsbG8=" // "hello" in base64
	testText   Tester = "hello"
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
	dataMsg := fmt.Sprintf(`{"hello": { "data": "%s" }}`, testBinary)
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
	callbackExecute := suite.parseCallbackExecute(t, callback)
	require.Len(t, callbackExecute.Success, 2)
	require.Len(t, callbackExecute.Error, 0)

	result1 := unmarshalExecute(t, callbackExecute.Success[0].Data).Data
	result2 := unmarshalExecute(t, callbackExecute.Success[1].Data).Data

	require.Equal(t, "hello", string(result1))
	require.Equal(t, "", string(result2))

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
	require.Len(t, callback.Success, 2)

	require.Equal(t,
		Callback{
			Success: [][]byte{
				[]byte(`{"amount":{"denom":"stake","amount":"100"}}`), // contracts get made with 100 coins.
				[]byte(`{"history":[]}`)},
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

	b, err := suite.RoundtripExecute(t, pathBA, &friend, []any{helloMsg})
	if err != nil {
		t.Fatal(err)
	}
	c, err := suite.RoundtripExecute(t, pathCA, &duplicate, []any{helloMsg})
	if err != nil {
		t.Fatal(err)
	}
	bCallbackExecute := suite.parseCallbackExecute(t, b)
	cCallbackExecute := suite.parseCallbackExecute(t, c)

	require.Equal(t, "", bCallbackExecute.Error)
	require.Equal(t, "", cCallbackExecute.Error)
	require.Equal(t, []byte(nil), bCallbackExecute.Success[0].Data)
	require.Equal(t, []byte(nil), cCallbackExecute.Success[0].Data)
	require.Equal(t, c, b)

	history := QueryHelloHistory(suite.ChainA.Chain, suite.ChainA.Tester)
	require.Len(t, history, 2)
	require.NotEqual(t, history[0], history[1])
}
