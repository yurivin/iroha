declare -a cmd_list=(
'./iroha_client_cli domain add --name="Soramitsu"'
'./iroha_client_cli domain add --name="Polkadot"'
'./iroha_client_cli domain add --name="Kusama"'
'./iroha_client_cli asset register --domain="Soramitsu" --name="XOR"'
'./iroha_client_cli asset register --domain="Polkadot" --name="DOT"'
'./iroha_client_cli asset register --domain="Kusama" --name="KSM"'
'./iroha_client_cli account register --domain="Soramitsu" --name="DEX Owner" --key="[120, 221, 193, 217, 83, 191, 157, 223, 1, 2, 205, 104, 209, 1, 180, 200, 29, 70, 220, 189, 221, 136, 221, 64, 31, 12, 44, 39, 179, 57, 141, 181]"'
'./iroha_client_cli dex initialize --domain="Soramitsu" --owner_account_id="DEX Owner@Soramitsu" --base_asset_id="XOR#Soramitsu"'
'./iroha_client_cli dex token_pair --domain="Soramitsu" create --base_asset_id="XOR#Soramitsu" --target_asset_id="DOT#Polkadot"'
'./iroha_client_cli dex token_pair --domain="Soramitsu" create --base_asset_id="XOR#Soramitsu" --target_asset_id="KSM#Kusama"'
'./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="DOT#Polkadot" create'
'./iroha_client_cli dex xyk_pool --domain="Soramitsu" --base_asset_id="XOR#Soramitsu" --target_asset_id="KSM#Kusama" create'
'./iroha_client_cli account register --domain="Soramitsu" --name="User A" --key="[162, 172, 183, 13, 229, 237, 8, 113, 177, 22, 100, 41, 174, 202, 106, 25, 216, 241, 18, 226, 77, 138, 250, 103, 10, 16, 194, 56, 21, 198, 90, 148]"'
'./iroha_client_cli asset mint --account_id="User A@Soramitsu" --id="XOR#Soramitsu" --quantity="12000"'
'./iroha_client_cli asset mint --account_id="User A@Soramitsu" --id="DOT#Polkadot" --quantity="4000"'
'./iroha_client_cli asset mint --account_id="User A@Soramitsu" --id="KSM#Kusama" --quantity="3000"'
'./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="XOR#Soramitsu"'
'./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="DOT#Polkadot"'
'./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="KSM#Kusama"'
'./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="XYKPOOL XOR-Soramitsu/DOT-Polkadot#Soramitsu"'
'./iroha_client_cli account add_transfer_permission --id="User A@Soramitsu" --asset_id="XYKPOOL XOR-Soramitsu/KSM-Kusama#Soramitsu"'
'./iroha_client_cli account register --domain="Soramitsu" --name="User B" --key="[171, 23, 228, 169, 169, 132, 244, 86, 72, 152, 12, 41, 160, 86, 186, 81, 54, 241, 116, 40, 246, 106, 252, 36, 114, 156, 121, 228, 213, 136, 109, 153]"'
'./iroha_client_cli asset mint --account_id="User B@Soramitsu" --id="XOR#Soramitsu" --quantity="500"'
'./iroha_client_cli asset mint --account_id="User B@Soramitsu" --id="DOT#Polkadot" --quantity="500"'
'./iroha_client_cli account add_transfer_permission --id="User B@Soramitsu" --asset_id="XOR#Soramitsu"'
'./iroha_client_cli account add_transfer_permission --id="User B@Soramitsu" --asset_id="DOT#Polkadot"'
'./iroha_client_cli account add_transfer_permission --id="User B@Soramitsu" --asset_id="XYKPOOL XOR-Soramitsu/DOT-Polkadot#Soramitsu"'
'./iroha_client_cli account register --domain="Soramitsu" --name="User C" --key="[196, 239, 3, 91, 95, 202, 55, 187, 149, 152, 2, 30, 178, 165, 167, 193, 45, 239, 205, 216, 185, 213, 155, 161, 92, 147, 242, 254, 27, 112, 199, 189]"'
'./iroha_client_cli asset mint --account_id="User C@Soramitsu" --id="KSM#Kusama" --quantity="2000"'
'./iroha_client_cli account add_transfer_permission --id="User C@Soramitsu" --asset_id="KSM#Kusama"'
)

for i in "${!cmd_list[@]}"
do
  eval "${cmd_list[$i]}"
  printf "Done %s/%s\n" $((i+1)) ${#cmd_list[@]}
  sleep 1s
done
