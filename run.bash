#!/usr/bin/env bash
# shellcheck disable=SC2181
# shellcheck disable=SC2145
# shellcheck disable=SC2034

set -eu

FAUCET_PORT=8080
LINERA_SERVICE_PORT_A=8081
LINERA_SERVICE_PORT_B=8082
LINERA_SERVICE_PORT_C=8083
FLUTTER_WEB_PORT_A=5173
FLUTTER_WEB_PORT_B=5174
FLUTTER_WEB_PORT_C=5175
LINERA_MAX_PENDING_MESSAGES=100

# Add cargo bin directory to PATH (where linera is installed)
export PATH="$HOME/.cargo/bin:$PWD/target/debug:$PATH"

# Verify linera is available
if ! command -v linera &> /dev/null; then
    echo "Error: linera command not found!"
    echo "Please install Linera CLI tools first:"
    echo "  cargo install --locked linera-service@0.15.7"
    echo "  cargo install --locked linera-storage-service@0.15.7"
    exit 1
fi

# -----------------------------------------------------------------------------------------------------------------
# Connect to Testnet Conway
# -----------------------------------------------------------------------------------------------------------------

# Use Testnet Conway faucet instead of local network
FAUCET_URL=https://faucet.testnet-conway.linera.net/
GRAPHQL_URL=http://localhost:$LINERA_SERVICE_PORT_A

# Set temporary directory for wallets and storage (if not already set)
if [ -z "${LINERA_TMP_DIR:-}" ]; then
  export LINERA_TMP_DIR="${TMPDIR:-/tmp}/linera_testnet"
  mkdir -p "$LINERA_TMP_DIR"
fi

echo "Connecting to Testnet Conway at $FAUCET_URL"
echo "Using temporary directory: $LINERA_TMP_DIR"

PUBLIC_CHAIN_AMOUNT=1
PLAY_CHAIN_AMOUNT_FOR_EACH_PUBLIC_CHAIN=1
TOKEN_AMOUNT_TO_MINT=1000000000

export LINERA_WALLET_1="$LINERA_TMP_DIR/wallet_1.json"
export LINERA_KEYSTORE_1="$LINERA_TMP_DIR/keystore_1.json"
export LINERA_STORAGE_1="rocksdb:$LINERA_TMP_DIR/client_1.db"

export LINERA_WALLET_2="$LINERA_TMP_DIR/wallet_2.json"
export LINERA_KEYSTORE_2="$LINERA_TMP_DIR/keystore_2.json"
export LINERA_STORAGE_2="rocksdb:$LINERA_TMP_DIR/client_2.db"

export LINERA_WALLET_3="$LINERA_TMP_DIR/wallet_3.json"
export LINERA_KEYSTORE_3="$LINERA_TMP_DIR/keystore_3.json"
export LINERA_STORAGE_3="rocksdb:$LINERA_TMP_DIR/client_3.db"

# ----------------------------------------------------------
# [FUNCTION] Initiate New Wallet from Faucet
# ----------------------------------------------------------

initiate_new_wallet_from_faucet() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: initiate_new_wallet_from_faucet <Wallet_Number>"
    exit 1
  fi

  # Check if keystore already exists
  WALLET_VAR="LINERA_KEYSTORE_$1"
  KEYSTORE_PATH="${!WALLET_VAR}"
  
  if [ -f "$KEYSTORE_PATH" ]; then
    echo "Keystore for wallet $1 already exists at $KEYSTORE_PATH"
    echo "Skipping wallet initialization (using existing wallet)"
    return 0
  fi

  linera --with-wallet "$1" wallet init --faucet "$FAUCET_URL"
  if [ $? -ne 0 ]; then
      echo "Initiate New Wallet from Faucet failed. Exiting..."
      exit 1
  fi
}

# ----------------------------------------------------------
# [FUNCTION] Open Chain from Faucet
# ----------------------------------------------------------

open_chain_from_faucet() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: open_chain_from_faucet <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" wallet request-chain --faucet "$FAUCET_URL"
  if [ $? -ne 0 ]; then
      echo "Open Chain from Faucet failed. Exiting..."
      exit 1
  fi
}

# ----------------------------------------------------------
# Create Initial Default Wallet and User Wallet
# ----------------------------------------------------------

INITIATE_WALLET_1=$(initiate_new_wallet_from_faucet 1)

OPEN_NEW_DEFAULT_WALLET_1=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET_1"
DEFAULT_CHAIN_ID_A=${StringArray[0]}

linera --with-wallet 1 sync && linera --with-wallet 1 query-balance

sleep 1

OPEN_NEW_USER_WALLET_A_1=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_1"
USER_CHAIN_ID_A_1=${StringArray[0]}

OPEN_NEW_USER_WALLET_A_2=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_2"
USER_CHAIN_ID_A_2=${StringArray[0]}

OPEN_NEW_USER_WALLET_A_3=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_3"
USER_CHAIN_ID_A_3=${StringArray[0]}

OPEN_NEW_USER_WALLET_A_4=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_4"
USER_CHAIN_ID_A_4=${StringArray[0]}

OPEN_NEW_USER_WALLET_A_5=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_5"
USER_CHAIN_ID_A_5=${StringArray[0]}

OPEN_NEW_USER_WALLET_A_6=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_6"
USER_CHAIN_ID_A_6=${StringArray[0]}

OPEN_NEW_USER_WALLET_A_7=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_7"
USER_CHAIN_ID_A_7=${StringArray[0]}

OPEN_NEW_USER_WALLET_A_8=$(open_chain_from_faucet 1)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_A_8"
USER_CHAIN_ID_A_8=${StringArray[0]}

sleep 1

linera --with-wallet 1 sync && linera --with-wallet 1 query-balance

sleep 1

INITIATE_WALLET_2=$(initiate_new_wallet_from_faucet 2)

OPEN_NEW_DEFAULT_WALLET_2=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET_2"
DEFAULT_CHAIN_ID_B=${StringArray[0]}

linera --with-wallet 2 sync && linera --with-wallet 2 query-balance

sleep 1

OPEN_NEW_USER_WALLET_B_1=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_1"
USER_CHAIN_ID_B_1=${StringArray[0]}

OPEN_NEW_USER_WALLET_B_2=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_2"
USER_CHAIN_ID_B_2=${StringArray[0]}

OPEN_NEW_USER_WALLET_B_3=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_3"
USER_CHAIN_ID_B_3=${StringArray[0]}

OPEN_NEW_USER_WALLET_B_4=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_4"
USER_CHAIN_ID_B_4=${StringArray[0]}

OPEN_NEW_USER_WALLET_B_5=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_5"
USER_CHAIN_ID_B_5=${StringArray[0]}

OPEN_NEW_USER_WALLET_B_6=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_6"
USER_CHAIN_ID_B_6=${StringArray[0]}

OPEN_NEW_USER_WALLET_B_7=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_7"
USER_CHAIN_ID_B_7=${StringArray[0]}

OPEN_NEW_USER_WALLET_B_8=$(open_chain_from_faucet 2)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_B_8"
USER_CHAIN_ID_B_8=${StringArray[0]}

sleep 1

linera --with-wallet 2 sync && linera --with-wallet 2 query-balance

sleep 1

INITIATE_WALLET_3=$(initiate_new_wallet_from_faucet 3)

OPEN_NEW_DEFAULT_WALLET_3=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_DEFAULT_WALLET_3"
DEFAULT_CHAIN_ID_C=${StringArray[0]}

linera --with-wallet 3 sync && linera --with-wallet 3 query-balance

sleep 1

OPEN_NEW_USER_WALLET_C_1=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_1"
USER_CHAIN_ID_C_1=${StringArray[0]}

OPEN_NEW_USER_WALLET_C_2=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_2"
USER_CHAIN_ID_C_2=${StringArray[0]}

OPEN_NEW_USER_WALLET_C_3=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_3"
USER_CHAIN_ID_C_3=${StringArray[0]}

OPEN_NEW_USER_WALLET_C_4=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_4"
USER_CHAIN_ID_C_4=${StringArray[0]}

OPEN_NEW_USER_WALLET_C_5=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_5"
USER_CHAIN_ID_C_5=${StringArray[0]}

OPEN_NEW_USER_WALLET_C_6=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_6"
USER_CHAIN_ID_C_6=${StringArray[0]}

OPEN_NEW_USER_WALLET_C_7=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_7"
USER_CHAIN_ID_C_7=${StringArray[0]}

OPEN_NEW_USER_WALLET_C_8=$(open_chain_from_faucet 3)
mapfile -t StringArray <<< "$OPEN_NEW_USER_WALLET_C_8"
USER_CHAIN_ID_C_8=${StringArray[0]}

sleep 1

linera --with-wallet 3 sync && linera --with-wallet 3 query-balance

sleep 1

# ----------------------------------------------------------
# Open New Chain IDs
# ----------------------------------------------------------
PUBLIC_CHAIN_IDS=()
for _ in $(seq 1 $PUBLIC_CHAIN_AMOUNT)
do
  OPEN_NEW_CHAIN=$(open_chain_from_faucet 1)
  mapfile -t StringArray <<< "$OPEN_NEW_CHAIN"
  NEW_CHAIN_ID=${StringArray[0]}
  PUBLIC_CHAIN_IDS+=("$NEW_CHAIN_ID")
  sleep 1
done

# Convert Chain IDs array to a JSON-formatted list
JSON_PUBLIC_CHAIN_IDS=$(printf '"%s",' "${PUBLIC_CHAIN_IDS[@]}")
JSON_PUBLIC_CHAIN_IDS="[${JSON_PUBLIC_CHAIN_IDS%,}]"

linera --with-wallet 1 sync && linera --with-wallet 1 query-balance

# Echo the values
echo ""
echo "PUBLIC_CHAIN_IDS: ${PUBLIC_CHAIN_IDS[@]}"
echo ""
echo "JSON_PUBLIC_CHAIN_IDS: $JSON_PUBLIC_CHAIN_IDS"
echo ""

# ----------------------------------------------------------
# [FUNCTION] Deploy Bankroll App
# ----------------------------------------------------------
deploy_bankroll_app() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: deploy_bankroll_app <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" --wait-for-outgoing-messages project publish-and-create . bankroll \
  --json-parameters "{
  \"master_chain\": \"$DEFAULT_CHAIN_ID_A\",
  \"bonus\": \"25000\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create Bankroll app failed. Exiting..."
      exit 1
  fi
}

BANKROLL_APP_ID=$(deploy_bankroll_app 1)
sleep 5

# ----------------------------------------------------------
# [FUNCTION] Deploy Poker App
# ----------------------------------------------------------
deploy_poker_app() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: deploy_poker_app <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" --wait-for-outgoing-messages project publish-and-create . poker \
  --required-application-ids "$BANKROLL_APP_ID" \
  --json-argument "10000" \
  --json-parameters "{
  \"master_chain\": \"$DEFAULT_CHAIN_ID_A\",
  \"public_chains\": $JSON_PUBLIC_CHAIN_IDS,
  \"bankroll\": \"$BANKROLL_APP_ID\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create Poker app failed. Exiting..."
      exit 1
  fi
}

POKER_APP_ID=$(deploy_poker_app 1)
sleep 5

# ----------------------------------------------------------
# [FUNCTION] Deploy Rummy App
# ----------------------------------------------------------
deploy_rummy_app() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: deploy_rummy_app <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" --wait-for-outgoing-messages project publish-and-create . rummy \
  --required-application-ids "$BANKROLL_APP_ID" \
  --json-argument "10000" \
  --json-parameters "{
  \"master_chain\": \"$DEFAULT_CHAIN_ID_A\",
  \"public_chains\": $JSON_PUBLIC_CHAIN_IDS,
  \"bankroll\": \"$BANKROLL_APP_ID\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create Rummy app failed. Exiting..."
      exit 1
  fi
}

RUMMY_APP_ID=$(deploy_rummy_app 1)
sleep 5

# ----------------------------------------------------------
# [FUNCTION] Deploy Roulette App
# ----------------------------------------------------------
deploy_roulette_app() {
  # Ensure Wallet_Number is passed as the first argument
  if [ -z "$1" ]; then
    echo "Error: Missing required parameter <Wallet_Number>. Usage: deploy_roulette_app <Wallet_Number>"
    exit 1
  fi

  linera --with-wallet "$1" --wait-for-outgoing-messages project publish-and-create . roulette \
  --required-application-ids "$BANKROLL_APP_ID" \
  --json-argument "10000" \
  --json-parameters "{
  \"master_chain\": \"$DEFAULT_CHAIN_ID_A\",
  \"public_chains\": $JSON_PUBLIC_CHAIN_IDS,
  \"bankroll\": \"$BANKROLL_APP_ID\"
  }"
  if [ $? -ne 0 ]; then
      echo "publish-and-create Roulette app failed. Exiting..."
      exit 1
  fi
}

ROULETTE_APP_ID=$(deploy_roulette_app 1)
sleep 5

# ----------------------------------------------------------
# Loop through each ChainID to create Play Chains for each Public Chain
# ----------------------------------------------------------

echo ""
echo "------------------------------------------------"
echo "1 | Create Play Chains for each Public Chain"
echo "------------------------------------------------"
echo ""

# Associative array to store player arrays by chain_id
declare -A PLAY_CHAIN_ID_COLLECTION

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  # Open New Chain IDs
  # Each Play Chain will have its own designated Public Chain
  PLAY_CHAIN_IDS=()
  for _ in $(seq 1 $PLAY_CHAIN_AMOUNT_FOR_EACH_PUBLIC_CHAIN)
  do
    OPEN_NEW_CHAIN=$(open_chain_from_faucet 1)
    mapfile -t StringArray <<< "$OPEN_NEW_CHAIN"
    NEW_CHAIN_ID=${StringArray[0]}
    PLAY_CHAIN_IDS+=("$NEW_CHAIN_ID")
    sleep 2
  done

  # Store the array as a space-separated string in the associative array
  PLAY_CHAIN_ID_COLLECTION["$PUBLIC_CHAIN_ID"]="${PLAY_CHAIN_IDS[*]}"

  linera --with-wallet 1 sync && linera --with-wallet 1 query-balance
  sleep 2
done

# ----------------------------------------------------------
# Running Node Service in Background
# ----------------------------------------------------------
echo "Running Node Service in Background..."
sleep 5

linera --max-pending-message-bundles $LINERA_MAX_PENDING_MESSAGES --with-wallet 1 service --port $LINERA_SERVICE_PORT_A &
SERVICE_PID_A=$!
echo "Node service A started with PID $SERVICE_PID_A"
sleep 5

linera --max-pending-message-bundles $LINERA_MAX_PENDING_MESSAGES --with-wallet 2 service --port $LINERA_SERVICE_PORT_B &
SERVICE_PID_B=$!
echo "Node service B started with PID $SERVICE_PID_B"
sleep 5

linera --max-pending-message-bundles $LINERA_MAX_PENDING_MESSAGES --with-wallet 3 service --port $LINERA_SERVICE_PORT_C &
SERVICE_PID_C=$!
echo "Node service C started with PID $SERVICE_PID_C"
sleep 5

# ----------------------------------------------------------
# Loop through each argument to AddPlayChain to each Public Chain (for Poker)
# ----------------------------------------------------------
echo ""
echo "------------------------------------------------"
echo "2 | Add Play Chains to each Public Chain (Poker)"
echo "------------------------------------------------"
echo ""

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  echo "AddPlayChain - Processing ChainID: $PUBLIC_CHAIN_ID"
  IFS=' ' read -r -a PLAY_CHAIN_IDS <<< "${PLAY_CHAIN_ID_COLLECTION[$PUBLIC_CHAIN_ID]}"

  for PLAY_CHAIN in "${PLAY_CHAIN_IDS[@]}"; do
    echo "PLAY_CHAIN: $PLAY_CHAIN"

    # Build the GraphQL mutation
    MUTATION="mutation { addPlayChain ( targetPublicChain: \\\"$PUBLIC_CHAIN_ID\\\", playChainId: \\\"$PLAY_CHAIN\\\" ) }"

    # Send request
    curl -s -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID_A/applications/$POKER_APP_ID" \
      -H "Content-Type: application/json" \
      -d "{\"query\":\"$MUTATION\"}" \
      | jq .

    sleep 2
  done
done

# ----------------------------------------------------------
# Loop through each argument to AddPlayChain to each Public Chain (for Rummy)
# ----------------------------------------------------------
echo ""
echo "------------------------------------------------"
echo "3 | Add Play Chains to each Public Chain (Rummy)"
echo "------------------------------------------------"
echo ""

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  echo "AddPlayChain - Processing ChainID: $PUBLIC_CHAIN_ID"
  IFS=' ' read -r -a PLAY_CHAIN_IDS <<< "${PLAY_CHAIN_ID_COLLECTION[$PUBLIC_CHAIN_ID]}"

  for PLAY_CHAIN in "${PLAY_CHAIN_IDS[@]}"; do
    echo "PLAY_CHAIN: $PLAY_CHAIN"

    # Build the GraphQL mutation
    MUTATION="mutation { addPlayChain ( targetPublicChain: \\\"$PUBLIC_CHAIN_ID\\\", playChainId: \\\"$PLAY_CHAIN\\\" ) }"

    # Send request
    curl -s -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID_A/applications/$RUMMY_APP_ID" \
      -H "Content-Type: application/json" \
      -d "{\"query\":\"$MUTATION\"}" \
      | jq .

    sleep 2
  done
done

# ----------------------------------------------------------
# Loop through each argument to AddPlayChain to each Public Chain (for Roulette)
# ----------------------------------------------------------
echo ""
echo "------------------------------------------------"
echo "4 | Add Play Chains to each Public Chain (Roulette)"
echo "------------------------------------------------"
echo ""

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  echo "AddPlayChain - Processing ChainID: $PUBLIC_CHAIN_ID"
  IFS=' ' read -r -a PLAY_CHAIN_IDS <<< "${PLAY_CHAIN_ID_COLLECTION[$PUBLIC_CHAIN_ID]}"

  for PLAY_CHAIN in "${PLAY_CHAIN_IDS[@]}"; do
    echo "PLAY_CHAIN: $PLAY_CHAIN"

    # Build the GraphQL mutation
    MUTATION="mutation { addPlayChain ( targetPublicChain: \\\"$PUBLIC_CHAIN_ID\\\", playChainId: \\\"$PLAY_CHAIN\\\" ) }"

    # Send request
    curl -s -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID_A/applications/$ROULETTE_APP_ID" \
      -H "Content-Type: application/json" \
      -d "{\"query\":\"$MUTATION\"}" \
      | jq .

    sleep 2
  done
done

# ----------------------------------------------------------
# Loop through each argument to MintToken to each Public Chain
# ----------------------------------------------------------
echo ""
echo "------------------------------------------------"
echo "5 | MintToken to each Public Chain"
echo "------------------------------------------------"
echo ""

for PUBLIC_CHAIN_ID in "${PUBLIC_CHAIN_IDS[@]}"; do
  echo "MintToken - Processing ChainID: $PUBLIC_CHAIN_ID"

  # Build the GraphQL mutation
  MUTATION="mutation { mintToken ( chainId: \\\"$PUBLIC_CHAIN_ID\\\", amount: \\\"$TOKEN_AMOUNT_TO_MINT\\\" ) }"

  # Send request
  curl -s -X POST "$GRAPHQL_URL/chains/$DEFAULT_CHAIN_ID_A/applications/$POKER_APP_ID" \
    -H "Content-Type: application/json" \
    -d "{\"query\":\"$MUTATION\"}" \
    | jq .

  sleep 2
done

# -----------------------------------------------------------------------------------------------------------------
# Create web server directories from master build
# -----------------------------------------------------------------------------------------------------------------

echo "Creating web server directories from master build..."

# Copy master build contents to each player directory
mkdir -p frontend/web_a frontend/web_b frontend/web_c
cp -r frontend/dist/. frontend/web_a 2>/dev/null || mkdir -p frontend/web_a
cp -r frontend/dist/. frontend/web_b 2>/dev/null || mkdir -p frontend/web_b
cp -r frontend/dist/. frontend/web_c 2>/dev/null || mkdir -p frontend/web_c

echo "✓ Web directories created"
echo ""

# -----------------------------------------------------------------------------------------------------------------
# Generate config.json for frontend A
# -----------------------------------------------------------------------------------------------------------------

PLAYER_A_NODE_URL=http://localhost:$LINERA_SERVICE_PORT_A

echo "Generating config.json for frontend A..."

jq -n \
  --arg nodeServiceURL "$PLAYER_A_NODE_URL" \
  --arg pokerAppId "$POKER_APP_ID" \
  --arg rummyAppId "$RUMMY_APP_ID" \
  --arg rouletteAppId "$ROULETTE_APP_ID" \
  --arg bankrollAppId "$BANKROLL_APP_ID" \
  --arg conwayDefaultChain "$DEFAULT_CHAIN_ID_A" \
  --arg conwayUserChain1 "$USER_CHAIN_ID_A_1" \
  --arg conwayUserChain2 "$USER_CHAIN_ID_A_2" \
  --arg conwayUserChain3 "$USER_CHAIN_ID_A_3" \
  --arg conwayUserChain4 "$USER_CHAIN_ID_A_4" \
  --arg conwayUserChain5 "$USER_CHAIN_ID_A_5" \
  --arg conwayUserChain6 "$USER_CHAIN_ID_A_6" \
  --arg conwayUserChain7 "$USER_CHAIN_ID_A_7" \
  --arg conwayUserChain8 "$USER_CHAIN_ID_A_8" \
  '{
    nodeServiceURL: $nodeServiceURL,
    pokerAppId: $pokerAppId,
    rummyAppId: $rummyAppId,
    rouletteAppId: $rouletteAppId,
    bankrollAppId: $bankrollAppId,
    defaultChain: $conwayDefaultChain,
    userChain1: $conwayUserChain1,
    userChain2: $conwayUserChain2,
    userChain3: $conwayUserChain3,
    userChain4: $conwayUserChain4,
    userChain5: $conwayUserChain5,
    userChain6: $conwayUserChain6,
    userChain7: $conwayUserChain7,
    userChain8: $conwayUserChain8
  }' > "frontend/web_a/config.json"

echo "✓ Config for Player A created at frontend/web_a/config.json"
echo ""

# -----------------------------------------------------------------------------------------------------------------
# Generate config.json for frontend B
# -----------------------------------------------------------------------------------------------------------------

PLAYER_B_NODE_URL=http://localhost:$LINERA_SERVICE_PORT_B

echo "Generating config.json for frontend B..."

jq -n \
  --arg nodeServiceURL "$PLAYER_B_NODE_URL" \
  --arg pokerAppId "$POKER_APP_ID" \
  --arg rummyAppId "$RUMMY_APP_ID" \
  --arg rouletteAppId "$ROULETTE_APP_ID" \
  --arg bankrollAppId "$BANKROLL_APP_ID" \
  --arg conwayDefaultChain "$DEFAULT_CHAIN_ID_A" \
  --arg conwayUserChain1 "$USER_CHAIN_ID_B_1" \
  --arg conwayUserChain2 "$USER_CHAIN_ID_B_2" \
  --arg conwayUserChain3 "$USER_CHAIN_ID_B_3" \
  --arg conwayUserChain4 "$USER_CHAIN_ID_B_4" \
  --arg conwayUserChain5 "$USER_CHAIN_ID_B_5" \
  --arg conwayUserChain6 "$USER_CHAIN_ID_B_6" \
  --arg conwayUserChain7 "$USER_CHAIN_ID_B_7" \
  --arg conwayUserChain8 "$USER_CHAIN_ID_B_8" \
  '{
    nodeServiceURL: $nodeServiceURL,
    pokerAppId: $pokerAppId,
    rummyAppId: $rummyAppId,
    rouletteAppId: $rouletteAppId,
    bankrollAppId: $bankrollAppId,
    defaultChain: $conwayDefaultChain,
    userChain1: $conwayUserChain1,
    userChain2: $conwayUserChain2,
    userChain3: $conwayUserChain3,
    userChain4: $conwayUserChain4,
    userChain5: $conwayUserChain5,
    userChain6: $conwayUserChain6,
    userChain7: $conwayUserChain7,
    userChain8: $conwayUserChain8
  }' > "frontend/web_b/config.json"

echo "✓ Config for Player B created at frontend/web_b/config.json"
echo ""

# -----------------------------------------------------------------------------------------------------------------
# Generate config.json for frontend C
# -----------------------------------------------------------------------------------------------------------------

PLAYER_C_NODE_URL=http://localhost:$LINERA_SERVICE_PORT_C

echo "Generating config.json for frontend C..."

jq -n \
  --arg nodeServiceURL "$PLAYER_C_NODE_URL" \
  --arg pokerAppId "$POKER_APP_ID" \
  --arg rummyAppId "$RUMMY_APP_ID" \
  --arg rouletteAppId "$ROULETTE_APP_ID" \
  --arg bankrollAppId "$BANKROLL_APP_ID" \
  --arg conwayDefaultChain "$DEFAULT_CHAIN_ID_A" \
  --arg conwayUserChain1 "$USER_CHAIN_ID_C_1" \
  --arg conwayUserChain2 "$USER_CHAIN_ID_C_2" \
  --arg conwayUserChain3 "$USER_CHAIN_ID_C_3" \
  --arg conwayUserChain4 "$USER_CHAIN_ID_C_4" \
  --arg conwayUserChain5 "$USER_CHAIN_ID_C_5" \
  --arg conwayUserChain6 "$USER_CHAIN_ID_C_6" \
  --arg conwayUserChain7 "$USER_CHAIN_ID_C_7" \
  --arg conwayUserChain8 "$USER_CHAIN_ID_C_8" \
  '{
    nodeServiceURL: $nodeServiceURL,
    pokerAppId: $pokerAppId,
    rummyAppId: $rummyAppId,
    rouletteAppId: $rouletteAppId,
    bankrollAppId: $bankrollAppId,
    defaultChain: $conwayDefaultChain,
    userChain1: $conwayUserChain1,
    userChain2: $conwayUserChain2,
    userChain3: $conwayUserChain3,
    userChain4: $conwayUserChain4,
    userChain5: $conwayUserChain5,
    userChain6: $conwayUserChain6,
    userChain7: $conwayUserChain7,
    userChain8: $conwayUserChain8
  }' > "frontend/web_c/config.json"

echo "✓ Config for Player C created at frontend/web_c/config.json"
echo ""

# -----------------------------------------------------------------------------------------------------------------
# Casino is READY
# -----------------------------------------------------------------------------------------------------------------

echo "-----------------------------------------------------------"
echo ""
echo "Linera Casino READY!"
echo ""
echo "visit http://localhost:5173 to play as Player 1"
echo "visit http://localhost:5174 to play as Player 2"
echo "visit http://localhost:5175 to play as Player 3"
echo ""
echo "Games available: Poker, Rummy, Roulette"
echo ""
echo "-----------------------------------------------------------"

# -----------------------------------------------------------------------------------------------------------------
# Build and run your frontend, if any
# -----------------------------------------------------------------------------------------------------------------

echo "Starting Player 1 web server on port 5173..."
cd frontend/web_a
npx http-server . -p $FLUTTER_WEB_PORT_A --cors -c0 --no-dotfiles &
WEB_SERVER_PID_A=$!

cd ../web_b
echo "Starting Player 2 web server on port 5174..."
npx http-server . -p $FLUTTER_WEB_PORT_B --cors -c0 --no-dotfiles &
WEB_SERVER_PID_B=$!

cd ../web_c
echo "Starting Player 3 web server on port 5175..."
npx http-server . -p $FLUTTER_WEB_PORT_C --cors -c0 --no-dotfiles

