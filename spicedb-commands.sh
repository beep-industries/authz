#!/bin/bash

# Setup context
spice context set local localhost:50051 foobar --insecure

# List all server relationships
spice relationship read server

# Check all permissions for owner
SERVER_ID="f8303518-4601-49d5-a2a3-28cdcb48e8e3"
USER_ID="af4028d5-cb24-44e8-b40b-90d2ea4c9500"

for perm in admin manage view send_message view_channel manage_message attach_files manage_webhooks manage_role view_role manage_nicknames change_nickname manage_channels; do
    echo -n "$perm: "
    spice permission check "server:$SERVER_ID" "$perm" "user:$USER_ID" 2>/dev/null
done
