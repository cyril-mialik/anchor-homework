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
    cargo test
