#!/bin/bash

POS_BITS=8
LENGTH_BITS=4

mkdir -p enc-files
mkdir -p dec-files

source_file="test-files/$1"
archive="enc-files/$1.lzss"
decoded="dec-files/$1"

echo -n "Encoding..."
# time RUST_BACKTRACE=1 RUST_LOG=debug cargo run -- encode $source_file $archive -c $LENGTH_BITS -s $POS_BITS -o > output.log 2>&1
time cargo run -- encode $source_file $archive -c $LENGTH_BITS -s $POS_BITS -o
echo -n "Decoding..."
# time RUST_BACKTRACE=1 RUST_LOG=debug cargo run -- decode $archive $decoded -o > dec-output.log 2>&1
time RUST_BACKTRACE=1 cargo run -- decode $archive $decoded -o 

echo "Original: "
ls -l $source_file
sha256sum $source_file
echo "Decoded: "
ls -l $decoded
sha256sum $decoded
echo "Archive: "
ls -l $archive