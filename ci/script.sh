set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo build
        cargo test --tests
        return
    fi

    xargo build --target $TARGET
    xargo test --target $TARGET --examples
}

main
