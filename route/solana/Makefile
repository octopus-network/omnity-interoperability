#!/usr/bin/make

.DEFAULT_GOAL: help

help: ## Show this help
	@printf "\033[33m%s:\033[0m\n" 'Available commands'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[32m%-18s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

# ----------------------------------------------------------------------------------------------------------------------

.PHONY: start
start: ## Start the canisters
	dfx stop
	RUST_BACKTRACE=1 dfx start --clean --background > dfx.out 2>&1

.PHONY: build
build: ## Build the canisters
	dfx build solana_route
	./scripts/build.sh

.PHONY: deploy
deploy: ## deploy the canisters
	./scripts/deploy.sh

.PHONY: init
init: ## init test data
	./scripts/init.sh

.PHONY: test
test: ## Run solana canister tests
	./scripts/test.sh

.PHONY: clean
clean: ## Cleanup
	dfx stop
	cargo clean
	rm -rf .dfx

%::
	@true
