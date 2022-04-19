NEAR_CONTRACT_BUILDER_IMAGE=nearprotocol/contract-builder
SESSION_DIR=session_vault
SESSION_BUILDER_NAME=build_session_vault
SESSION_RELEASE=session_vault

RFLAGS="-C link-arg=-s"

build: session vault token

docker-session:
	$(call create_builder,${SESSION_BUILDER_NAME},${SESSION_DIR})
	$(call start_builder,${SESSION_BUILDER_NAME})
	$(call setup_builder,${SESSION_BUILDER_NAME})
	cp target/wasm32-unknown-unknown/release/${SESSION_RELEASE}.wasm res/${SESSION_RELEASE}_release.wasm

session: 
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p session_vault --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/session_vault.wasm ./res/session_vault.wasm

vault: 
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p vault --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/vault.wasm ./res/vault.wasm

token: 
	rustup target add wasm32-unknown-unknown
	RUSTFLAGS=$(RFLAGS) cargo build -p test_token --target wasm32-unknown-unknown --release
	mkdir -p res
	cp target/wasm32-unknown-unknown/release/test_token.wasm ./res/test_token.wasm

test-session: session token
	cd session_vault && RUSTFLAGS=$(RFLAGS) cargo test

define create_builder 
	docker ps -a | grep $(1) || docker create \
		--mount type=bind,source=${PWD},target=/host \
		--cap-add=SYS_PTRACE --security-opt seccomp=unconfined \
		--name=$(1) \
		-w /host/$(2) \
		-e RUSTFLAGS='-C link-arg=-s' \
		-it \
		${NEAR_CONTRACT_BUILDER_IMAGE} \
		/bin/bash
endef

define start_builder
	docker ps | grep $(1) || docker start $(1) 
endef

define setup_builder
	docker exec $(1) /bin/bash _rust_setup.sh 
endef

define remove_builder
	docker stop $(1) && sleep 3 && docker rm $(1) 
endef

define release_wasm
	cp ${PWD}/target/wasm32-unknown-unknown/release/$(1).wasm ${PWD}/res/$(1)_release.wasm
endef