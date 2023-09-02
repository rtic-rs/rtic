#!/bin/sh

version=$(cargo metadata --format-version 1 --no-deps --offline | jq -r '.packages[] | select(.name == "rtic") | .version')

echo $version