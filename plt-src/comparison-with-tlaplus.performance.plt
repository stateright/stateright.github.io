set title 'TLC/Stateright Model Checking Time' font ",14"
set xlabel 'states'
set ylabel 'sec'
set key left top
set logscale x
set logscale y
set term svg

cd 'plt-src/'
set output '../md-src/comparison-with-tlaplus.performance.svg'
plot 'comparison-with-tlaplus.performance.dat' u 2:3 w lp ps 1.5 pt 5 ti 'TLC', \
     ''                                        u 2:4 w lp ps 1.5 pt 7 ti 'Stateright'
