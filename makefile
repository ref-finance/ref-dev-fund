RFLAGS="-C link-arg=-s"

build: session vault token

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