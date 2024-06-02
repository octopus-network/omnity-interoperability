include ./*.mk

.PHONY: all start_dfx rust build install_hub_on_local perf_hub_on_local perf_on_ic clean
all: clean start_dfx rust build install_hub_on_local perf_hub_on_local perf_on_ic

start_dfx: clean
	dfx start --clean --background > dfx.out 2>&1
	sleep 5;

rust: start_dfx
	$(call build,../)

build: rust

install_hub_on_local: build
	ic-repl ./install_hub_on_local.sh;

perf_hub_on_local: install_hub_on_local
	./test_hub_on_local.sh;

build_ic: clean
	cd ../;
	
	$(call build_on_ic,../)

perf_ic: build_ic
	ic-repl -r ic ./install_on_ic.sh;
	./perf.sh --network ic;

clean:
	dfx stop -q >/dev/null 2>&1; 
	cargo clean;
	rm -f ./process;
	rm -f ./perf.md;
	rm -f ./*.out;
	# rm -f *.svg;
	# rm -rvf ../.dfx/;

env:
	(printf "> * "; dfx --version) >> perf.md; \
	(printf "> * "; rustc --version) >> perf.md; \
	(printf "> * "; ic-repl --version) >> perf.md; \
	(printf "> * "; ic-wasm --version) >> perf.md; \
