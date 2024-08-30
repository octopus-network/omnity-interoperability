#!/usr/bin/make

.DEFAULT_GOAL: help

help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

.PHONY: start
start: ## Start ic localnet
	dfx stop
	RUST_BACKTRACE=1 dfx start --clean --background > dfx.out 2>&1

.PHONY: build
build: start ## Build the solana route canister
	# dfx canister create solana_route
	# dfx build solana_route
	./scripts/build.sh

.PHONY: deploy
deploy: ## Deploy the canisters(omnity_hub,schnorr_canister,ic-solana-provider and solana_route)
	./scripts/deploy.sh

.PHONY: init
init: ## Init test data
	./scripts/init.sh

.PHONY: test
test: ## Run e2e test
	./scripts/test.sh

.PHONY: stress
stress: ## Run stress test on localnet
	./scripts/stress_localnet.sh

.PHONY: clean
clean: ## Cleanup
	dfx stop
	cargo clean
	rm -rf .dfx

%::
	@true
