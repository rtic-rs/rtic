set -euxo pipefail

main() {
    if [ $TARGET != x86_64-unknown-linux-gnu ]; then
        rustup target add $TARGET
    fi

    mkdir qemu
    curl -L https://github.com/japaric/qemu-bin/raw/master/14.04/qemu-system-arm-2.12.0 > qemu/qemu-system-arm
    chmod +x qemu/qemu-system-arm

    # install mdbook
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- --git rust-lang-nursery/mdbook --tag v0.2.1

    pip install linkchecker --user
}

main
