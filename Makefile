.PHONY: all firmware kkcli rest vault-ui vault test test-rest

all: firmware kkcli rest vault-ui vault

test:
	cargo test --manifest-path projects/keepkey-rest/Cargo.toml --all-features

firmware:
	$(MAKE) -C firmware

kkcli:
	cd projects/kkcli && cargo build && target/debug/kkcli server

vault:
	cd projects/vault-v2 && bun i && tauri dev
