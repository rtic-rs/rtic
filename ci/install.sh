set -euxo pipefail

main() {
    # these are not needed for doc builds
    if [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_PULL_REQUEST != false ]; then
        if [ $TARGET = x86_64-unknown-linux-gnu ]; then
            ( cd .. && cargo install microamp-tools --version 0.1.0-alpha.3 -f )
            rustup target add thumbv6m-none-eabi thumbv7m-none-eabi
        fi

        rustup target add $TARGET

        mkdir qemu
        curl -L https://github.com/japaric/qemu-bin/raw/master/14.04/qemu-system-arm-2.12.0 > qemu/qemu-system-arm
        chmod +x qemu/qemu-system-arm

        pip install linkchecker --user
    fi

    # install mdbook
    curl -LSfs https://japaric.github.io/trust/install.sh | \
        sh -s -- --git rust-lang-nursery/mdbook --tag v0.3.1
}

main
