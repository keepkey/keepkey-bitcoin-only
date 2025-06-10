.PHONY: all firmware kkcli rest vault-ui vault

all: firmware kkcli rest vault-ui vault

firmware:
	$(MAKE) -C firmware

kkcli:
	cargo build --manifest-path projects/kkcli/Cargo.toml

rest:
	cargo build --manifest-path keepkey-rest/Cargo.toml

vault-ui:
	cd vault-ui && npm install && npm run build

vault:
	cd vault && cargo tauri build
