set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo build
        cargo test --test cfail
        return
    fi

    case $TARGET in
        thumbv7em-none-eabi*)
            cargo check --target $TARGET --features cm7-r0p1
            cargo check --target $TARGET --features cm7-r0p1 --examples
        ;;
    esac

    cargo check --target $TARGET
    cargo check --target $TARGET --examples
}

main
