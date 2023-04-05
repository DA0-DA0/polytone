package strangelove

import (
	"context"
	"encoding/json"
	"errors"
	"strconv"
	"testing"

	"github.com/strangelove-ventures/interchaintest/v4"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos"
	"github.com/strangelove-ventures/interchaintest/v4/chain/cosmos/wasm"
	"github.com/strangelove-ventures/interchaintest/v4/ibc"
	"github.com/strangelove-ventures/interchaintest/v4/testreporter"
	"github.com/strangelove-ventures/interchaintest/v4/testutil"

	"go.uber.org/zap/zaptest"
)

const PATH = "juno-juno"

type Suite struct {
	t        *testing.T
	reporter *testreporter.RelayerExecReporter
	ctx      context.Context

	ChainA SuiteChain
	ChainB SuiteChain

	Relayer ibc.Relayer
	PathAB  string
}

type SuiteChain struct {
	Ibc    ibc.Chain
	Cosmos *cosmos.CosmosChain
	User   *ibc.Wallet

	Note   string
	Voice  string
	Tester string
}

func NewSuite(t *testing.T) Suite {
	ctx := context.Background()

	factory := interchaintest.NewBuiltinChainFactory(
		zaptest.NewLogger(t),
		[]*interchaintest.ChainSpec{
			{
				Name:      "juno",
				ChainName: "juno1",
				Version:   "latest",
				ChainConfig: ibc.ChainConfig{
					Denom:          "ujuno",
					GasAdjustment:  2.0,
					GasPrices:      "0.00ujuno",
					EncodingConfig: wasm.WasmEncoding(),
				},
			},
			{
				Name:      "juno",
				ChainName: "juno2",
				Version:   "latest",
				ChainConfig: ibc.ChainConfig{
					Denom:          "ujuno",
					GasAdjustment:  2.0,
					GasPrices:      "0.00ujuno",
					EncodingConfig: wasm.WasmEncoding(),
				},
			},
		},
	)
	chains, err := factory.Chains(t.Name())
	if err != nil {
		t.Fatal(err)
	}
	ibcA, ibcB := chains[0], chains[1]
	cosmosA, cosmosB := ibcA.(*cosmos.CosmosChain), ibcB.(*cosmos.CosmosChain)

	dockerClient, dockerNetwork := interchaintest.DockerSetup(t)
	relayer := interchaintest.
		NewBuiltinRelayerFactory(
			ibc.CosmosRly,
			zaptest.NewLogger(t),
		).
		Build(t, dockerClient, dockerNetwork)

	interchain := interchaintest.NewInterchain().
		AddChain(ibcA).
		AddChain(ibcB).
		AddRelayer(relayer, "cosmos-rly").
		AddLink(interchaintest.InterchainLink{
			Chain1:  ibcA,
			Chain2:  ibcB,
			Relayer: relayer,
			Path:    PATH,
		})
	reporter := testreporter.NewNopReporter().RelayerExecReporter(t)
	err = interchain.Build(ctx, reporter, interchaintest.InterchainBuildOptions{
		TestName:         t.Name(),
		Client:           dockerClient,
		NetworkID:        dockerNetwork,
		SkipPathCreation: false,
	})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		_ = interchain.Close()
	})
	err = relayer.StartRelayer(ctx, reporter, PATH)
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(func() {
		err := relayer.StopRelayer(ctx, reporter)
		if err != nil {
			t.Logf("couldn't stop relayer: %s", err)
		}
	})

	users := interchaintest.GetAndFundTestUsers(
		t,
		ctx,
		"default",
		int64(100_000_000),
		ibcA,
		ibcB,
	)
	userA, userB := users[0], users[1]

	suite := Suite{
		t:        t,
		reporter: reporter,
		ctx:      ctx,
		ChainA: SuiteChain{
			Ibc:    ibcA,
			Cosmos: cosmosA,
			User:   userA,
		},
		ChainB: SuiteChain{
			Ibc:    ibcB,
			Cosmos: cosmosB,
			User:   userB,
		},
		Relayer: relayer,
	}

	suite.SetupChain(&suite.ChainA, suite.ChainA.User)
	suite.SetupChain(&suite.ChainB, suite.ChainB.User)

	return suite
}

func (s *Suite) SetupChain(chain *SuiteChain, user *ibc.Wallet) {
	cc := chain.Cosmos
	noteId, err := cc.StoreContract(s.ctx, user.KeyName, "../wasms/polytone_note.wasm")
	if err != nil {
		s.t.Fatal(err)
	}
	_, err = cc.StoreContract(s.ctx, user.KeyName, "../wasms/polytone_voice.wasm")
	if err != nil {
		s.t.Fatal(err)
	}
	proxyId, err := cc.StoreContract(s.ctx, user.KeyName, "../wasms/polytone_proxy.wasm")
	if err != nil {
		s.t.Fatal(err)
	}

	_, err = cc.StoreContract(s.ctx, user.KeyName, "../wasms/polytone_tester.wasm")
	if err != nil {
		s.t.Fatal(err)
	}

	_, err = strconv.Atoi(proxyId)
	if err != nil {
		s.t.Fatal(err)
	}

	chain.Note = s.Instantiate(cc, user, noteId, NoteInstantiate{})

	// instantiating voice needs:
	// https://github.com/strangelove-ventures/interchaintest/issues/482

	// chain.Voice = s.Instantiate(cc, user, voiceId, VoiceInstantiate{
	// 	ProxyCodeId: uint64(proxyUint),
	// 	BlockMaxGas: 100_000_000,
	// })
	// chain.Tester = s.Instantiate(cc, user, testerId, TesterInstantiate{})
	return
}

func (s *Suite) Instantiate(chain *cosmos.CosmosChain, user *ibc.Wallet, codeId string, msg any) string {
	str, err := json.Marshal(msg)
	if err != nil {
		s.t.Fatal(err)
	}
	s.t.Logf("🙏 %s", str)
	address, err := chain.InstantiateContract(s.ctx, user.KeyName, codeId, string(str), true)
	if err != nil {
		s.t.Fatal(err)
	}
	return address
}

func (s *Suite) CreateChannel(initModule string, tryModule string, initChain, tryChain *SuiteChain) (initChannel, tryChannel string, err error) {
	initStartChannels := s.QueryOpenChannels(initChain)

	err = s.Relayer.CreateChannel(s.ctx, s.reporter, PATH, ibc.CreateChannelOptions{
		SourcePortName: "wasm." + initModule,
		DestPortName:   "wasm." + tryModule,
		Order:          ibc.Unordered,
		Version:        "polytone-1",
	})
	if err != nil {
		return
	}
	err = testutil.WaitForBlocks(s.ctx, 10, initChain.Ibc, tryChain.Ibc)
	if err != nil {
		return
	}

	initChannels := s.QueryOpenChannels(initChain)

	if len(initChannels) == len(initStartChannels) {
		err = errors.New("no new channels created")
		return
	}

	initChannel = initChannels[len(initChannels)-1].ChannelID
	tryChannel = initChannels[len(initChannels)-1].Counterparty.ChannelID
	return
}

const CHANNEL_STATE_OPEN = "STATE_OPEN"
const CHANNEL_STATE_TRY = "STATE_TRYOPEN"
const CHANNEL_STATE_INIT = "STATE_INIT"

func (s *Suite) QueryChannelsInState(chain *SuiteChain, state string) []ibc.ChannelOutput {
	channels, err := s.Relayer.GetChannels(s.ctx, s.reporter, chain.Ibc.Config().ChainID)
	if err != nil {
		s.t.Fatal(err)
	}
	openChannels := []ibc.ChannelOutput{}
	for index := range channels {
		if channels[index].State == "STATE_OPEN" {
			openChannels = append(openChannels, channels[index])
		}
	}
	return openChannels
}

func (s *Suite) QueryOpenChannels(chain *SuiteChain) []ibc.ChannelOutput {
	return s.QueryChannelsInState(chain, CHANNEL_STATE_OPEN)
}
