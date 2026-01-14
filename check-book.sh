#!/bin/sh

set -e

cd book/en/
mdbook build
cd ../../

cargo doc --features thumbv7-backend

mkdir -p book-target/book/
cp -r book/en/book/ book-target/book/en/
cp LICENSE-* book-target/book/en
cp -r target/doc/ book-target/api/

lychee -v --offline --format detailed --exclude '.*/\d{1}' --root-dir book-target book-target/book/en/

rm -rf book-target/
