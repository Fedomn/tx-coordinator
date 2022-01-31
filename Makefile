.PHONY: test run build check clean fix release

default: test

test: fix
	cargo test --all-features

run:
	cargo run

build:
	cargo build

check:
	cargo check --all
	cargo deny check

clean:
	cargo clean

fix:
	cargo fix --allow-dirty --allow-staged
	cargo fmt -- --check
	cargo clippy --all-targets --all-features --tests --benches -- -D warnings

doc:
	cargo doc --all-features --no-deps

release: test doc
	cargo build --release

simulate:
	RUST_LOG=tx_coordinator=debug cargo run -- --cfg ./tests/cfg.toml --dir ./tests/sqlfiles