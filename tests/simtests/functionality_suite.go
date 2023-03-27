package simtests

import (
	"testing"

	"github.com/CosmWasm/wasmd/x/wasm/ibctesting"
	sdk "github.com/cosmos/cosmos-sdk/types"
	minttypes "github.com/cosmos/cosmos-sdk/x/mint/types"
	channeltypes "github.com/cosmos/ibc-go/v4/modules/core/04-channel/types"
	sdkibctesting "github.com/cosmos/ibc-go/v4/testing"
	"github.com/stretchr/testify/require"
)

type Chain struct {
	Chain  *ibctesting.TestChain
	Note   sdk.AccAddress
	Voice  sdk.AccAddress
	Tester sdk.AccAddress
}

type Suite struct {
	Coordinator *ibctesting.Coordinator
	ChainA      Chain
	ChainB      Chain
	ChainC      Chain
}

func SetupChain(t *testing.T, c *ibctesting.Coordinator, index int) Chain {
	chain := c.GetChain(sdkibctesting.GetChainID(index))
	chain.StoreCodeFile("../wasms/polytone_note.wasm")
	chain.StoreCodeFile("../wasms/polytone_voice.wasm")
	chain.StoreCodeFile("../wasms/polytone_proxy.wasm")
	chain.StoreCodeFile("../wasms/polytone_tester.wasm")

	note := Instantiate(t, chain, 1, NoteInstantiate{})
	voice := Instantiate(t, chain, 2, VoiceInstantiate{ProxyCodeId: 3})
	tester := Instantiate(t, chain, 4, TesterInstantiate{})

	return Chain{
		Chain:  chain,
		Note:   note,
		Voice:  voice,
		Tester: tester,
	}
}

func NewSuite(t *testing.T) Suite {
	coordinator := ibctesting.NewCoordinator(t, 3)
	chainA := SetupChain(t, coordinator, 0)
	chainB := SetupChain(t, coordinator, 1)
	chainC := SetupChain(t, coordinator, 2)

	return Suite{
		Coordinator: coordinator,
		ChainA:      chainA,
		ChainB:      chainB,
		ChainC:      chainC,
	}
}

func ChannelConfig(port string) *sdkibctesting.ChannelConfig {
	return &sdkibctesting.ChannelConfig{
		PortID:  port,
		Version: "polytone",
		Order:   channeltypes.UNORDERED,
	}
}

func (s *Suite) SetupPath(
	chainA,
	chainB *Chain,
) *ibctesting.Path {
	aPort := chainA.Chain.ContractInfo(chainA.Note).IBCPortID
	bPort := chainB.Chain.ContractInfo(chainB.Voice).IBCPortID

	path := ibctesting.NewPath(chainA.Chain, chainB.Chain)
	path.EndpointA.ChannelConfig = ChannelConfig(aPort)
	path.EndpointB.ChannelConfig = ChannelConfig(bPort)
	s.Coordinator.Setup(path)
	return path
}

func (c *Chain) MintBondedDenom(t *testing.T, to sdk.AccAddress) {
	chain := c.Chain
	bondDenom := chain.App.StakingKeeper.BondDenom(chain.GetContext())
	coins := sdk.NewCoins(sdk.NewCoin(bondDenom, sdk.NewInt(100000000)))

	err := chain.App.BankKeeper.MintCoins(chain.GetContext(), minttypes.ModuleName, coins)
	require.NoError(t, err)

	err = chain.App.BankKeeper.SendCoinsFromModuleToAccount(chain.GetContext(), minttypes.ModuleName, to, coins)
	require.NoError(t, err)
}

func (s *Suite) RoundtripExecute(t *testing.T, path *ibctesting.Path, account *Account, msgs []any) (Callback, error) {
	msg := NoteExecuteMsg{
		Msgs:           msgs,
		TimeoutSeconds: 100,
		Callback: &CallbackRequest{
			Receiver: account.SuiteChain.Tester.String(),
			Msg:      "aGVsbG8K",
		},
	}
	return s.RoundtripMessage(t, path, account, NoteExecute{
		Execute: &msg,
	})
}

func (s *Suite) RoundtripQuery(t *testing.T, path *ibctesting.Path, account *Account, msgs []any) (Callback, error) {
	msg := NoteQuery{
		Msgs:           msgs,
		TimeoutSeconds: 100,
		Callback: CallbackRequest{
			Receiver: account.SuiteChain.Tester.String(),
			Msg:      "aGVsbG8K",
		},
	}
	return s.RoundtripMessage(t, path, account, NoteExecute{
		Query: &msg,
	})
}

func (s *Suite) RoundtripMessage(t *testing.T, path *ibctesting.Path, account *Account, msg NoteExecute) (Callback, error) {
	wasmMsg := account.WasmExecute(&account.SuiteChain.Note, msg)
	if _, err := account.Send(t, wasmMsg); err != nil {
		return Callback{}, err
	}
	if err := s.Coordinator.RelayAndAckPendingPackets(path); err != nil {
		return Callback{}, err
	}
	callbacks := QueryCallbackHistory(account.Chain, account.SuiteChain.Tester)
	callback := callbacks[len(callbacks)-1]
	require.Equal(t, account.Address.String(), callback.Initiator)
	require.Equal(t, "aGVsbG8K", callback.InitiatorMsg)
	return callback.Result, nil
}
