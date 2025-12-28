BINARY := "fbscreenshot"
TARGET_GLIBC := "2.17"

targets := replace(
    "aarch64-unknown-linux-gnu.GLIBC \
    armv7-unknown-linux-gnueabihf.GLIBC",
    "GLIBC",
    TARGET_GLIBC
)

deps:
    cargo install --locked cargo-zigbuild
    for target in {{targets}}; do \
        rustup target add "${target%.{{TARGET_GLIBC}}}"; \
    done

release:
    for target in {{targets}}; do \
        cargo zigbuild --release --target "$target"; \
    done

dist: release
    mkdir -p dist
    for target in {{targets}}; do \
        target="${target%.{{TARGET_GLIBC}}}"; \
        outfile="dist/{{BINARY}}-$target"; \
        cp target/"$target"/release/{{BINARY}} "$outfile"; \
        xz "$outfile"; \
    done

clean:
    rm -rf dist
    cargo clean

