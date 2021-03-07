#!/bin/sh

dot -Tsvg -Gsize='8,8!' -o md-src/getting-started.states.svg dot-src/getting-started.states.dot
mdbook build

