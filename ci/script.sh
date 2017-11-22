set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo build
        cargo test --test cfail
        return
    fi

    xargo build --target $TARGET
    xargo check --target $TARGET --examples
}

main
