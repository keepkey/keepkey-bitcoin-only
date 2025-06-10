.PHONY: all firmware kkcli rest vault-ui vault test test-rest

all: firmware kkcli rest vault-ui vault

test:
	cargo test --manifest-path projects/keepkey-rest/Cargo.toml --all-features

firmware:
	$(MAKE) -C firmware

kkcli:
	cd projects/kkcli && cargo build && target/debug/kkcli server

rest:
	cargo build --manifest-path keepkey-rest/Cargo.toml

test-rest:
	./skills/test-rest.sh

vault-ui:
	cd projects/vault-ui && bun install && bun run dev

vault:
	cd projects/vault && bun i && cargo tauri build
