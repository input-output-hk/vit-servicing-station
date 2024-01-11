#!/bin/bash

# ---------------------------------------------------------------
# Entrypoint script for voting-node container
# ---------------------------------------------------------------
#
# This script serves as the entrypoint for the jormungandr node.
#
# It expects the following environment variables to be set except where noted:
#
# CONFIG_PATH - The path to the server configuration file
# DATABASE_URL - The path to the local sqlite database
# VERSION - The version to use for the service (affects what /api/vit-version returns)
# ENV (optional) - The target environment. Used for fetching the database from S3.
# FUND (optional) - The fund name. Used for fetching artifacts from S3.
# ARTIFACT_BUCKET (optional) - The S3 bucket where the database and genesis block is stored.
# DATABASE_PATH (optional) - The path to the local sqlite database. If not set, the database will be fetched from S3.
# DATABASE_VERSION (optional) - The database version. Used for fetching the database from S3. If not set, the "default" version will be used.
# GENESIS_PATH (optional) - The path to the genesis block. If not set, the genesis block will be fetched from S3.
# GENESIS_VERSION (optional) - The genesis version. Used for fetching the genesis block from S3. If not set, the "default" version will be used.
# DEBUG_SLEEP (optional) - If set, the script will sleep for the specified number of seconds before starting the node.
# ---------------------------------------------------------------

# Enable strict mode
set +x
set -o errexit
set -o pipefail
set -o nounset
set -o functrace
set -o errtrace
set -o monitor
set -o posix
shopt -s dotglob

check_env_vars() {
    local env_vars=("$@")

    # Iterate over the array and check if each variable is set
    for var in "${env_vars[@]}"; do
        echo "Checking $var"
        if [ -z "${!var+x}" ]; then
            echo ">>> Error: $var is required and not set."
            exit 1
        fi
    done
}

debug_sleep() {
    if [ -n "${DEBUG_SLEEP:-}" ]; then
        echo "DEBUG_SLEEP is set. Sleeping for ${DEBUG_SLEEP} seconds..."
        sleep "${DEBUG_SLEEP}"
    fi
}

fetch_database() {
    local bucket=$1
    local env=$2
    local fund=$3
    local version=$4
    local path=$5

    echo ">>> Fetching database from S3 using the following parameters..."
    echo "Bucket: $bucket"
    echo "Environment: $env"
    echo "Fund: $fund"
    echo "Version: ${version}"

    mkdir -p "$(dirname "$path")"
    fetcher --bucket "$bucket" artifact -e "$env" -f "$fund" -t "vit" -v "${version}" "$path"
}

fetch_genesis() {
    local bucket=$1
    local env=$2
    local fund=$3
    local version=$4
    local path=$5

    echo ">>> Fetching genesis block from S3 using the following parameters..."
    echo "Bucket: $bucket"
    echo "Environment: $env"
    echo "Fund: $fund"
    echo "Version: ${version}"

    mkdir -p "$(dirname "$path")"
    fetcher --bucket "$bucket" artifact -e "$env" -f "$fund" -t "genesis" -v "${version}" "$path"
}

echo ">>> Starting entrypoint script..."

REQUIRED_ENV=(
    "CONFIG_PATH"
    "DATABASE_PATH"
    "VERSION"
)
echo ">>> Checking required env vars..."
check_env_vars "${REQUIRED_ENV[@]}"

# Verify the config exists
if [[ ! -f "$CONFIG_PATH" ]]; then
    echo "ERROR: configuration file does not exist at: $CONFIG_PATH"
    echo ">>> Aborting..."
    exit 1
fi

# Check if the local database exists
if [[ ! -f "$DATABASE_PATH" ]]; then
    echo ">>> No database provided. Attempting to fetch from S3..."

    REQUIRED_ENV=(
        "ENV"
        "FUND"
        "ARTIFACT_BUCKET"
    )
    echo ">>> Checking required env vars for fetching from S3..."
    check_env_vars "${REQUIRED_ENV[@]}"

    fetch_database "$ARTIFACT_BUCKET" "$ENV" "$FUND" "${DATABASE_VERSION:=}" "$DATABASE_PATH"
fi

# Check if the local genesis block exists
if [[ ! -f "${GENESIS_PATH:=}" ]]; then
    echo ">>> No genesis block provided. Attempting to fetch from S3..."

    REQUIRED_ENV=(
        "ENV"
        "FUND"
        "ARTIFACT_BUCKET"
    )
    echo ">>> Checking required env vars for fetching from S3..."
    check_env_vars "${REQUIRED_ENV[@]}"

    fetch_genesis "$ARTIFACT_BUCKET" "$ENV" "$FUND" "${GENESIS_VERSION:=}" "$GENESIS_PATH"
fi

echo ">>> Using the following parameters:"
echo "Config file: $CONFIG_PATH"
echo "Database: $DATABASE_PATH"
echo "Database hash (SHA256): $(sha256sum "$DATABASE_PATH" | awk '{ print $1 }')"
echo "Genesis block: $GENESIS_PATH"
echo "Genesis block hash (SHA256): $(sha256sum "$GENESIS_PATH" | awk '{ print $1 }')"
echo "Version: $VERSION"

args+=()
args+=("--in-settings-file" "$CONFIG_PATH")
args+=("--service-version" "$VERSION")

# Sleep if DEBUG_SLEEP is set
debug_sleep

echo ">>> Starting server..."

export DATABASE_URL="$DATABASE_PATH"
exec "/app/vit-servicing-station-server" "${args[@]}"
