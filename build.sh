#!/bin/sh

dot -Tsvg -Gsize='8,8!' -o md-src/getting-started.states.svg dot-src/getting-started.states.dot
gnuplot plt-src/comparison-with-tlaplus.performance.plt
mdbook build

