#!/usr/bin/env bash
set -e

# Start up local testing infrastructure

docker compose pull
docker compose build --pull
docker compose up
