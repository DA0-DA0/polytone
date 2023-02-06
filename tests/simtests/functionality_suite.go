package simtests

import (
	"testing"

	"github.com/CosmWasm/wasmd/x/wasm/ibctesting"
	sdk "github.com/cosmos/cosmos-sdk/types"
	channeltypes "github.com/cosmos/ibc-go/v4/modules/core/04-channel/types"
	sdkibctesting "github.com/cosmos/ibc-go/v4/testing"
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
	coordinator := ibctesting.NewCoordinator(t, 2)
	chainA := SetupChain(t, coordinator, 0)
	chainB := SetupChain(t, coordinator, 1)

	return Suite{
		Coordinator: coordinator,
		ChainA:      chainA,
		ChainB:      chainB,
	}
}

func ChannelConfig(port string) *sdkibctesting.ChannelConfig {
	return &sdkibctesting.ChannelConfig{
		PortID:  port,
		Version: "polytone",
		Order:   channeltypes.UNORDERED,
	}
}

func SetupPath(
	coordinator *ibctesting.Coordinator,
	chainA,
	chainB *ibctesting.TestChain,
	note,
	voice sdk.AccAddress) *ibctesting.Path {
	aPort := chainA.ContractInfo(note).IBCPortID
	bPort := chainB.ContractInfo(voice).IBCPortID

	path := ibctesting.NewPath(chainA, chainB)
	path.EndpointA.ChannelConfig = ChannelConfig(aPort)
	path.EndpointB.ChannelConfig = ChannelConfig(bPort)
	coordinator.Setup(path)
	return path
}
