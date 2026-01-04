#!/bin/bash
set -e

echo "Building programs..."
anchor build

echo "Generating clients for all programs..."

echo "Generating dac clients..."
npx codama run --all -c codama-dac.json

echo "Build and client generation completed successfully!"

