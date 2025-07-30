#!/usr/bin/env bash

set -ex

workspace_crates=(
    "rec"
)

for crate in "${workspace_crates[@]}"; do
   echo "--- $crate"
   cargo package -p $crate --allow-dirty
   cargo publish -p $crate --allow-dirty
done