package simtests

import (
	"encoding/json"
	"testing"

	"github.com/CosmWasm/wasmd/x/wasm/ibctesting"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type NoteInstantiate struct {
}

type VoiceInstantiate struct {
	ProxyCodeId uint64 `json:"proxy_code_id,string"`
}

type TesterInstantiate struct {
}

type NoteExecute struct {
	Query   *NoteQuery      `json:"query,omitempty"`
	Execute *NoteExecuteMsg `json:"execute,omitempty"`
}

type NoteQuery struct {
	Msgs           []any           `json:"msgs"`
	TimeoutSeconds uint64          `json:"timeout_seconds,string"`
	Callback       CallbackRequest `json:"callback"`
}

type NoteExecuteMsg struct {
	Msgs           []any            `json:"msgs"`
	TimeoutSeconds uint64           `json:"timeout_seconds,string"`
	Callback       *CallbackRequest `json:"callback,omitempty"`
}

type PolytoneMessage struct {
	Query   *PolytoneQuery   `json:"query,omitempty"`
	Execute *PolytoneExecute `json:"execute,omitempty"`
}

type PolytoneQuery struct {
	Msgs []any `json:"msgs"`
}

type PolytoneExecute struct {
	Msgs []any `json:"msgs"`
}

type CallbackRequest struct {
	Receiver string `json:"receiver"`
	Msg      string `json:"msg"`
}

type CallbackMessage struct {
	Initiator    string   `json:"initiator"`
	InitiatorMsg string   `json:"initiator_msg"`
	Result       Callback `json:"result"`
}

type Callback struct {
	Success []string `json:"success,omitempty"`
	Error   string   `json:"error,omitempty"`
}

type TesterQuery struct {
	History NoteInstantiate `json:"history"`
}

type HistoryResponse struct {
	History []CallbackMessage `json:"history"`
}

func Instantiate(t *testing.T, chain *ibctesting.TestChain, codeId uint64, msg any) sdk.AccAddress {
	instantiate, err := json.Marshal(msg)
	if err != nil {
		t.Fatal(err)
	}
	return chain.InstantiateContract(codeId, instantiate)
}

func QueryHistory(chain *ibctesting.TestChain, tester sdk.AccAddress) []CallbackMessage {
	bytes, err := json.Marshal(TesterQuery{
		History: NoteInstantiate{},
	})
	if err != nil {
		panic(err)
	}
	res, err := chain.App.WasmKeeper.QuerySmart(chain.GetContext(), tester, bytes)
	if err != nil {
		panic(err)
	}
	var response HistoryResponse
	json.Unmarshal(res, &response)
	return response.History
}
