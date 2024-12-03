#!/usr/bin/env just --justfile

run-watch:
  bacon shuttle

validate exercise:
  cch24-validator -u http://localhost:8010 {{exercise}}