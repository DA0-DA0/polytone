package simtests

import (
	"encoding/base64"
	"testing"

	w "github.com/CosmWasm/wasmvm/types"
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

	path := SetupPath(
		suite.Coordinator,
		suite.ChainA.Chain,
		suite.ChainB.Chain,
		suite.ChainA.Note,
		suite.ChainB.Voice,
	)

	// Execute two messages, the first of which uses
	// polytone-tester to set some data in the transaction
	// response, and the second of which sets the proxy's staking
	// rewards receiver address to the voice address on the remote
	// chain.

	accountA := GenAccount(t, suite.ChainA.Chain)
	dataMsg := `{"hello": { "data": "aGVsbG8K" }}`
	dataCosmosMsg := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Execute: &w.ExecuteMsg{
				ContractAddr: suite.ChainB.Tester.String(),
				// base64.StdEncoding.EncodeToString()
				Msg:   []byte(dataMsg),
				Funds: []w.Coin{},
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

	encodedMsg := base64.StdEncoding.EncodeToString([]byte("the data is hello base64 encoded"))

	wasmMsg := accountA.WasmExecute(&suite.ChainA.Note, NoteExecute{
		Execute: &NoteExecuteMsg{
			Msgs:           []any{dataCosmosMsg, noDataCosmosMsg},
			TimeoutSeconds: 100,
			Callback: &CallbackRequest{
				Receiver: suite.ChainA.Tester.String(),
				Msg:      encodedMsg,
			},
		},
	})
	t.Log(wasmMsg)
	_, err := accountA.Send(t, wasmMsg)
	if err != nil {
		t.Fatal(err)
	}

	err = suite.Coordinator.RelayAndAckPendingPackets(path)
	if err != nil {
		t.Fatal(err)
	}

	callbackHistory := QueryHistory(suite.ChainA.Chain, suite.ChainA.Tester)
	require.Equal(t, []CallbackMessage{
		CallbackMessage{
			Initiator:    accountA.Address.String(),
			InitiatorMsg: encodedMsg,
			Result: Callback{
				Success: []string{"aGVsbG8K", ""},
			},
		},
	}, callbackHistory)

	balanceQuery := w.QueryRequest{
		Bank: &w.BankQuery{
			Balance: &w.BalanceQuery{
				Address: suite.ChainB.Note.String(),
				Denom:   suite.ChainB.Chain.App.StakingKeeper.BondDenom(suite.ChainB.Chain.GetContext()),
			},
		},
	}

	wasmMsg = accountA.WasmExecute(&suite.ChainA.Note, NoteExecute{
		Query: &NoteQuery{
			Msgs:           []any{balanceQuery},
			TimeoutSeconds: 100,
			Callback: CallbackRequest{
				Receiver: suite.ChainA.Tester.String(),
				Msg:      "",
			},
		},
	})
	t.Log(wasmMsg)
	_, err = accountA.Send(t, wasmMsg)
	if err != nil {
		t.Fatal(err)
	}

	err = suite.Coordinator.RelayAndAckPendingPackets(path)
	if err != nil {
		t.Fatal(err)
	}

	callbackHistory = QueryHistory(suite.ChainA.Chain, suite.ChainA.Tester)
	require.Equal(t,
		CallbackMessage{
			Initiator:    accountA.Address.String(),
			InitiatorMsg: "",
			Result: Callback{
				Success: []string{base64.StdEncoding.EncodeToString([]byte(`{"amount":{"denom":"stake","amount":"100"}}`))}, // contracts get made with 100 coins.
			},
		}, callbackHistory[1])

}
