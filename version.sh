#/bin/sh

set -e
set -u

find -name Cargo.toml -exec sed -i "s/stateright = \"[0-9.]*\"/stateright = \"$1\"/" {} \;
