#!/usr/bin/bash

echo ">>> Entering entrypoint script..."

# Verify the config exists
if [[ ! -f "$CONFIG_PATH" ]]; then
    echo "ERROR: configuration file does not exist at: $CONFIG_PATH"
    echo ">>> Aborting..."
    exit 1
fi

# Allow overriding vit-servicing-station-server binary
BIN_PATH=${BIN_PATH:=/app/vit-servicing-station-server}

echo ">>> Using the following parameters:"
echo "Config file: $CONFIG_PATH"

args+=()
args+=("--in-settings-file" "$CONFIG_PATH")
args+=("--service-version" "$VERSION")

echo ">>> Running servicing station..."
exec "$BIN_PATH" "''${args[@]}"
