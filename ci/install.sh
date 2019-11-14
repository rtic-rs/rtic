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

    install_crate mdbook 0.3.1
}

main
