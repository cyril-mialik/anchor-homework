default:
    @just --list

install:
    cd backend && npm install
    cd frontend && npm install

validator:
    solana-test-validator

build:
    anchor build

deploy:
    anchor deploy

init:
    cd backend && npm run init

backend:
    cd backend && npm run start

frontend:
    cd frontend && npm run dev

test:
    #!/usr/bin/env bash
    set -euo pipefail

    echo "Building programs for SBPF v2..."
    for program in programs/*/; do
        if [ -f "${program}Cargo.toml" ]; then
            echo "Building: $program"
            cargo build-sbf --arch v2 --manifest-path "${program}Cargo.toml"
        fi
    done

    echo "Running tests..."
    cargo test
