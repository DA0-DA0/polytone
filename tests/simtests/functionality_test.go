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

	path := suite.SetupDefaultPath(&suite.ChainA, &suite.ChainB)

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

	callbackExecute, err := suite.RoundtripExecute(t, path, &accountA, []any{dataCosmosMsg, noDataCosmosMsg})
	if err != nil {
		t.Fatal(err)
	}
	require.Len(t, callbackExecute.Success, 2)
	require.Len(t, callbackExecute.Error, 0)

	result1 := unmarshalExecute(t, callbackExecute.Success[0].Data).Data
	result2 := unmarshalExecute(t, callbackExecute.Success[1].Data).Data

	require.Equal(t, string(testText), string(result1))
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

	callbackQuery, err := suite.RoundtripQuery(t, path, &accountA, []any{balanceQuery, historyQuery})
	if err != nil {
		t.Fatal(err)
	}
	require.Len(t, callbackQuery.Success, 2)

	require.Equal(t,
		CallbackDataQuery{
			Success: [][]byte{
				[]byte(`{"amount":{"denom":"stake","amount":"100"}}`), // contracts get made with 100 coins.
				[]byte(`{"history":[]}`)},
		}, callbackQuery)
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

	pathCA := suite.SetupDefaultPath(&suite.ChainC, &suite.ChainA)
	pathBA := suite.SetupDefaultPath(&suite.ChainB, &suite.ChainA)

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

	require.Equal(t, "", b.Error)
	require.Equal(t, "", c.Error)
	require.Equal(t, []byte(nil), b.Success[0].Data)
	require.Equal(t, []byte(nil), c.Success[0].Data)
	require.Equal(t, c, b)

	history := QueryHelloHistory(suite.ChainA.Chain, suite.ChainA.Tester)
	require.Len(t, history, 2)
	require.NotEqual(t, history[0], history[1])
}

// Checks that connections between two of the same modules are not
// allowed. This checks that we are using the handshake logic, the
// other permutations of the handshake are tested in the
// polytone/handshake package.
func TestHandshakeBetweenSameModule(t *testing.T) {
	suite := NewSuite(t)

	aNote := suite.ChainA.QueryPort(suite.ChainA.Note)
	aVoice := suite.ChainA.QueryPort(suite.ChainA.Voice)
	bNote := suite.ChainB.QueryPort(suite.ChainB.Note)
	bVoice := suite.ChainB.QueryPort(suite.ChainB.Voice)

	_, err := suite.SetupPath(aNote, bNote, &suite.ChainA, &suite.ChainB)
	require.ErrorContains(t,
		err,
		"channel open try callback failed",
		"note <-/-> note",
	)
	// for reasons i do not understand, if the try step fails the
	// sequence number for the sending account does not get
	// incremented correctly. as a bandaid, this manually corrects.
	//
	// FIXME: why?
	suite.ChainB.Chain.SenderAccount.SetSequence(suite.ChainA.Chain.SenderAccount.GetSequence() + 1)

	_, err = suite.SetupPath(bVoice, aVoice, &suite.ChainB, &suite.ChainA)
	require.ErrorContains(t,
		err,
		"channel open try callback failed",
		"voice <-/-> voice",
	)
	suite.ChainA.Chain.SenderAccount.SetSequence(suite.ChainA.Chain.SenderAccount.GetSequence() + 1)

	_, err = suite.SetupPath(aVoice, bNote, &suite.ChainA, &suite.ChainB)
	require.NoError(t, err, "voice <- -> note")
}

// Executes a message on the note chain that will run out of gas on
// the voice chain and makes sure that an ACK + callback indicating
// that the out-of-gas error occured is returned.
func TestVoiceOutOfGas(t *testing.T) {
	suite := NewSuite(t)

	path := suite.SetupDefaultPath(&suite.ChainA, &suite.ChainB)

	accountA := GenAccount(t, &suite.ChainA)
	gasMsg := `{"run_out_of_gas":{}}`
	gasCosmosgMsg := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Execute: &w.ExecuteMsg{
				ContractAddr: suite.ChainB.Tester.String(),
				Msg:          []byte(gasMsg),
				Funds:        []w.Coin{},
			},
		},
	}

	callback, err := suite.RoundtripExecute(t, path, &accountA, []any{gasCosmosgMsg})

	// SDK codespace 11 is out-of-gas. See cosmos-sdk/types/errors/errors.go
	require.NoError(t, err, "out-of-gas should not error")
	require.Equal(t, CallbackDataExecute{
		Error: "codespace: sdk, code: 11",
	}, callback, "out-of-gas should return an ACK")
}

// Tests executing a message on the remote chain, checking the
// callback, and then executing another message.
//
// This tests that we correctly save proxies and reuse them upon
// another message being executed.
func TestMultipleMessages(t *testing.T) {
	suite := NewSuite(t)

	path := suite.SetupDefaultPath(&suite.ChainA, &suite.ChainB)

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
	require.NoError(t, err)

	result1 := unmarshalExecute(t, callback.Success[0].Data).Data
	result2 := unmarshalExecute(t, callback.Success[1].Data).Data
	require.Equal(t,
		[]string{string(testText), ""},
		[]string{string(result1), string(result2)})

	callback, err = suite.RoundtripExecute(t, path, &accountA, []any{dataCosmosMsg, noDataCosmosMsg})
	require.NoError(t, err)

	result1 = unmarshalExecute(t, callback.Success[0].Data).Data
	result2 = unmarshalExecute(t, callback.Success[1].Data).Data
	require.Equal(t,
		[]string{string(testText), ""},
		[]string{string(result1), string(result2)})
}

// A note may only ever connect to a single voice. This simplifies the
// API (as channel_id does not need to be specifed after a single
// handshake), and simplifies the protocol.
func TestOneVoicePerNote(t *testing.T) {
	suite := NewSuite(t)
	// connect note on A to voice on C. note should not connect
	// any additional connections.
	_ = suite.SetupDefaultPath(&suite.ChainA, &suite.ChainC)

	cPort := suite.ChainB.QueryPort(suite.ChainC.Voice)
	bPort := suite.ChainB.QueryPort(suite.ChainB.Voice)
	aPort := suite.ChainA.QueryPort(suite.ChainA.Note)
	_, err := suite.SetupPath(
		bPort,
		aPort,
		&suite.ChainB,
		&suite.ChainA,
	)
	require.ErrorContains(t,
		err,
		"contract is already paired with port ("+
			cPort+
			") on connection (connection-0), got port ("+
			bPort+
			") on connection (connection-1)",
		"two voices may not be connected to the same note",
	)
}

func TestInstantiateExecute(t *testing.T) {
	suite := NewSuite(t)

	path := suite.SetupDefaultPath(&suite.ChainA, &suite.ChainB)

	accountA := GenAccount(t, &suite.ChainA)
	msg, err := json.Marshal(TesterInstantiate{})
	require.NoError(t, err)
	initCosmosMsg := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Instantiate: &w.InstantiateMsg{
				CodeID: 4,
				Msg:    msg,
				Funds:  []w.Coin{},
				Label:  "test",
			},
		},
	}

	callback, err := suite.RoundtripExecute(t, path, &accountA, []any{initCosmosMsg})
	require.NoError(t, err)
	require.Empty(t, callback.Error, "callback should not error")
	response := unmarshalInstantiate(t, callback.Success[0].Data)

	// address should be: cosmos1ghd753shjuwexxywmgs4xz7x2q732vcnkm6h2pyv9s6ah3hylvrqa0dr5q
	// But because it can change in the future, we just check its not empty
	require.NotEmpty(t, response.Address, "address should not be empty")
}
