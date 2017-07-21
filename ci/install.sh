set -ex

main() {
    case $TARGET in
        thumbv*-none-eabi*)
            cargo install --list | grep xargo || \
                cargo install xargo
            rustup component list | grep 'rust-src.*installed' || \
                rustup component add rust-src
            ;;
    esac
}

main
