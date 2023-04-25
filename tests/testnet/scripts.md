### upload

`starsd tx wasm upload artifacts/polytone_tester.wasm --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 --node https://rpc.elgafar-1.stargaze-apis.com:443 --chain-id elgafar-1 --from tester -b block -y`

`junod tx wasm upload artifacts/polytone_voice.wasm --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.9 --node https://uni-rpc.reece.sh:443 --chain-id uni-6 --from tester -b block -y`

### init

Note

`starsd tx wasm init 2041 '{"block_max_gas": "75000000"}' --label "test_note" --admin stars1hec7cw4tn0xqjhwz674fpza0e3s7kdh35sk6cv --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 --node https://rpc.elgafar-1.stargaze-apis.com:443 --chain-id elgafar-1 --from tester -b block -y`

Tester

`starsd tx wasm init 2042 '{}' --label "test_note" --admin stars1hec7cw4tn0xqjhwz674fpza0e3s7kdh35sk6cv --gas-prices 0.025ustars --gas auto --gas-adjustment 1.9 --node https://rpc.elgafar-1.stargaze-apis.com:443 --chain-id elgafar-1 --from tester -b block -y`

Voice

`junod tx wasm init 1700 '{"proxy_code_id": "1702", "block_max_gas": "10000000"}' --label "test_voice" --admin juno1hec7cw4tn0xqjhwz674fpza0e3s7kdh3k7zu5p --gas-prices 0.025ujunox --gas auto --gas-adjustment 1.9 --node https://uni-rpc.reece.sh:443 --chain-id uni-6 --from tester -b block -y`

### Query for remote address from note

`starsd query wasm contract-state smart stars1tlhlkammrz8esaetxd3sycksuqqjatxg5qvpr05xp5lza4lvrs8qzulutw '{"remote_address": {"local_address": "stars1hec7cw4tn0xqjhwz674fpza0e3s7kdh35sk6cv"}}'`

### Hermes

Make sure you use the hermes config you set up (hermes.toml in this dir)

Confirm the config is correct

`hermes --config tests/testnet/hermes.toml health-check`

add a key with a mnemonic file

`hermes --config tests/testnet/hermes.toml keys add --chain elgafar-1 --mnemonic-file tests/testnet/test_mnemonic`

Create channel between 2 contracts with a new connection

`hermes --config tests/testnet/hermes.toml create channel --a-chain elgafar-1 --b-chain uni-6 --a-port wasm.stars1tlhlkammrz8esaetxd3sycksuqqjatxg5qvpr05xp5lza4lvrs8qzulutw --b-port wasm.juno1humhlky97ulwq8c9wwzt4nvf3kqjmd9lp80rhf6l2wu679gjx88sghhlqf --new-client-connection --channel-version polytone-1 --yes`

Start listen to the chains (And channels) you set in the config, make sure you only listen to the channels you opened!

`hermes --config tests/testnet/hermes.toml start`

relay packets only once, instead of listen to the chains, you can check once if there are packets to relay, and relay them (need to do it twice, once for packet, and once for the ack)

`hermes --config tests/testnet/hermes.toml clear`
