set -euxo pipefail

main() {
    case $TARGET in
        thumbv*-none-eabi*)
            cargo install --list | grep 'xargo v0.3.8' || \
                cargo install xargo --vers 0.3.8
            rustup component list | grep 'rust-src.*installed' || \
                rustup component add rust-src
            ;;
    esac
}

main
