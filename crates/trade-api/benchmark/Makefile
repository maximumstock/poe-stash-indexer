bench-local:
	vegeta attack -name trade-local -targets targets.local.txt | tee results.local.bin | vegeta report

bench:
	vegeta attack -name trade-prod -targets targets.prod.txt -rate 200 | tee results.prod.bin | vegeta report

plot-local:
	cat results.local.bin | vegeta plot > plot.local.html && chromium plot.local.html

plot:
	cat results.prod.bin | vegeta plot > plot.prod.html && chromium plot.prod.html
