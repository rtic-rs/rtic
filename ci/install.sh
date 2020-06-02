set -euxo pipefail

install_crate() {
    local pkg=$1 vers=$2

    cargo install --list | grep "$pkg v$vers" || ( cd .. && cargo install -f --vers $vers $pkg )
}

main() {
    # these are not needed for doc builds
    if [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_PULL_REQUEST != false ]; then
        if [ $TARGET = x86_64-unknown-linux-gnu ]; then
            install_crate microamp-tools 0.1.0-alpha.3
            rustup target add thumbv6m-none-eabi thumbv7m-none-eabi
        fi

        rustup target add $TARGET
        mkdir qemu
        curl -L https://github.com/japaric/qemu-bin/raw/master/14.04/qemu-system-arm-2.12.0 > qemu/qemu-system-arm
        chmod +x qemu/qemu-system-arm

        pip install linkchecker --user
    fi

    # Download binary mdbook and add to path
    curl -L https://github.com/rust-lang/mdBook/releases/download/v0.3.1/mdbook-v0.3.1-x86_64-unknown-linux-gnu.tar.gz > mdbook.tar.gz
    tar -xf mdbook.tar.gz
    mkdir -p mdbook-bin
    mv mdbook mdbook-bin/

    #install_crate mdbook 0.3.1
}

main
