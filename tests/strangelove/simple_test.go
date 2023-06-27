package strangelove

import (
	"testing"

	wasmdtypes "github.com/CosmWasm/wasmd/x/wasm/types"
	w "github.com/CosmWasm/wasmvm/types"
	"github.com/stretchr/testify/require"
)

// Tests that a simple message can be executed on the remote chain and
// return a callback.
func TestSimpleMessageExecution(t *testing.T) {
	suite := NewSuite(t)

	_, _, err := suite.CreateChannel(
		suite.ChainA.Note,
		suite.ChainB.Voice,
		&suite.ChainA,
		&suite.ChainB,
	)
	if err != nil {
		t.Fatal(err)
	}

	testerMsg := `{"hello": { "data": "aGVsbG8=" }}` // `hello` in base64
	message := w.CosmosMsg{
		Wasm: &w.WasmMsg{
			Execute: &w.ExecuteMsg{
				ContractAddr: suite.ChainB.Tester,
				Msg:          []byte(testerMsg),
				Funds:        []w.Coin{},
			},
		},
	}

	callback, err := suite.RoundtripExecute(suite.ChainA.Note, &suite.ChainA, message)
	var response wasmdtypes.MsgExecuteContractResponse
	response.Unmarshal(callback.Ok.Result[0].Data)

	require.Equal(t, "hello", string(response.Data), "single message should work")
	require.Len(t, callback.Ok.Result, 1, "a single message should cause a single response")
}
