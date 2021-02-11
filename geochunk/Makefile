# To build a static Linux binary package and publish it:
#
# 1. Update `version` in `Cargo.toml`.
# 2. Run `make upload`.

PKG_VERSION=$(shell cargo read-manifest | jq -r .version)
ZIP=geochunk-$(PKG_VERSION)-x86_64-linux.zip
EXPORT=geochunk_zip2010_500000.csv.sz

default: zip

upload: $(ZIP) $(EXPORT)
	fdyi aws s3 cp $(ZIP) s3://fdy-private-binaries/
	fdyi aws s3 cp $(EXPORT) s3://fdy-private-binaries/

zip: $(ZIP)

$(ZIP): build
	rm -f $(ZIP)
	zip -j $(ZIP) target/x86_64-unknown-linux-musl/release/geochunk

build:
	cargo build --target=x86_64-unknown-linux-musl --release

$(EXPORT): build
	geochunk export zip2010 500000 | szip > $(EXPORT)

clean:
	rm -f $(ZIP)

.PHONY: default upload zip build clean
