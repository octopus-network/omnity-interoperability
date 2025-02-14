#!/usr/bin/make

.DEFAULT_GOAL: help

help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

CLEAN_OPTION ?=

.PHONY: start
start: ## Start ic localnet
	dfx stop
	RUST_BACKTRACE=1 dfx start $(CLEAN_OPTION) --background > dfx.out 2>&1

.PHONY: new
new: CLEAN_OPTION=--clean
new: start ## Create and build a new sui route canister on localnet
	dfx canister create sui_route
	bash ./scripts/build.sh

.PHONY: build
build: start ## Build the sui route canister
	bash ./scripts/build.sh

.PHONY: did
did: ## Extract the did file
	bash ./scripts/did.sh

.PHONY: deploy
deploy: ## Deploy the canisters(omnity_hub,schnorr_canister,ic-sui-provider and sui_route)
	bash ./scripts/deploy.sh

.PHONY: upgrade
upgrade: did ## Deploy the canisters(omnity_hub,schnorr_canister,ic-sui-provider and sui_route)
	bash ./scripts/upgrade.sh

# .PHONY: test
# test: ## Run e2e test
# 	bash ./scripts/test.sh

# .PHONY: stress
# stress: ## Run stress test on localnet
# 	bash ./scripts/stress_localnet.sh

.PHONY: clean
clean: ## Cleanup
	dfx stop
	cargo clean
	rm -rf .dfx

%::
	@true
