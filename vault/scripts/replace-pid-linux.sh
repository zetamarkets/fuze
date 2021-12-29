#!/bin/bash
ROOT="$(git rev-parse --show-toplevel)/vault"

echo $ROOT

old_pubkey=$(solana address -k ./target/deploy/vault-keypair.json)
rm $ROOT/target/deploy/vault-keypair.json
solana-keygen new --outfile $ROOT/target/deploy/vault-keypair.json
new_pubkey=$(solana address -k ./target/deploy/vault-keypair.json)

echo "old_pubkey = $old_pubkey"
echo "new_pubkey = $new_pubkey"

sed -i "s/$old_pubkey/$new_pubkey/g" $ROOT/Anchor.toml
sed -i "s/$old_pubkey/$new_pubkey/g" $ROOT/programs/vault/src/lib.rs

cd $ROOT