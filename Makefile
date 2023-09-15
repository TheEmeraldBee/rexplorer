VERSION_FILE := Cargo.toml

clean:
	cargo clean
	git checkout "${VERSION_FILE}"

test:
	cargo test

build:
	cargo build

publish: test
	sed -i -r "s/0\.0\.0/${VERSION}/g" "${VERSION_FILE}" \
	&& cargo publish --allow-dirty
	
