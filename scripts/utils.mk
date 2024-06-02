define ic-wasm
	echo "Optimize with ic-wasm"; \
	for f in $(1)/*/*.wasm; do ic-wasm -o $$f $$f optimize O3 --keep-name-section; done
endef


define build_on_ic
	set -e; \
	cd $(1); \
	dfx build omnity_hub --network ic ; \
	$(call ic-wasm,.dfx/local/canisters/); \
	cd ..
endef


define build
	set -e; \
	cd $(1); \
	dfx canister create omnity_hub; \
	dfx ledger fabricate-cycles --t 100 --canister $$(dfx identity get-wallet); \
	dfx build omnity_hub; \
	$(call ic-wasm,.dfx/local/canisters/); \
	cd ..
endef
