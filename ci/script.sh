set -ex

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo test
        return
    fi

    cross build --target $TARGET
    cross build --target $TARGET --release
}

main
