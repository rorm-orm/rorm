#!/usr/bin/env bash

# This file is part of the unittest CI
# It should be executed in the `rorm-sample` directory
# It builds the CLI and the project and executes it
# It expects two env variables, CONFIG_FILE (mandatory) and CARGO_FLAGS (optional)

set -e

function run() {
  echo "Executing: $@"
  $@
}

if [ -z ${CONFIG_FILE+x} ]; then
  echo "Mandatory env variable 'CONFIG_FILE' is not set!" >> /dev/stderr
  exit 1
fi

if [ -z ${CARGO_FLAGS+x} ]; then
  echo "No cargo flags have been set"
else
  echo "Cargo flags: '${CARGO_FLAGS}'"
fi

if [[ -f "$(which rorm-cli)" ]]; then
  run
else
  run cargo install rorm-cli --path ../rorm-cli
fi

if [ -z ${CARGO_FLAGS+x} ]; then
  run cargo build
else
  run cargo build "${CARGO_FLAGS}"
fi

run rorm-cli migrate --database-config "${CONFIG_FILE}" --log-sql

if [ -z ${CARGO_FLAGS+x} ]; then
  run cargo run -F rorm-main
else
  run cargo run -F rorm-main "${CARGO_FLAGS}"
fi

run rorm-cli make-migrations

run rorm-cli migrate --database-config "${CONFIG_FILE}" --log-sql

run cargo run -- --help

if [ -z ${CARGO_FLAGS+x} ]; then
  RUST_LOG=rorm=debug run cargo run -- "${CONFIG_FILE}"
else
  RUST_LOG=rorm=debug run cargo run "${CARGO_FLAGS}" -- "${CONFIG_FILE}"
fi
