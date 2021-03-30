#!/bin/sh

dot -Tsvg -Gsize='8,8!' -o md-src/getting-started.states.svg dot-src/getting-started.states.dot
gnuplot plt-src/comparison-with-tlaplus.performance.plt
mdbook build
mkdir -p docs/2021-03-30
cp -r html-src/2021-03-30/dist docs/2021-03-30/dist
cp -r html-src/2021-03-30/plugin docs/2021-03-30/plugin
cp -r html-src/2021-03-30/index.html docs/2021-03-30/
cp html-src/2021-03-30/*.{jpg,png,svg} docs/2021-03-30/
