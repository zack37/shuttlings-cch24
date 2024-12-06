#!/usr/bin/env just --justfile

fmt:
    cargo +nightly fmt

lint:
    cargo clippy

lint-fix:
    cargo clippy --fix --allow-dirty

run:
    shuttle run --port 8010

run-watch:
    bacon shuttle

validate exercise:
    cch24-validator -u http://localhost:8010 {{exercise}}

deploy:
    shuttle deploy --name shuttlings-cch24
