set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo build
        cargo test --test cfail
        return
    fi

    # examples that don't require the timer-queue feature
    local examples=(
        async
        empty
        interrupt
    )

    case $TARGET in
        thumbv7em-none-eabi*)
            cargo check --target $TARGET --features cm7-r0p1
            for ex in ${examples[@]}; do
                cargo check --target $TARGET --features cm7-r0p1 --example $ex
            done

            cargo check timer-queue --target $TARGET --features "cm7-r0p1 timer-queue"
            cargo check --target $TARGET --features "cm7-r0p1 timer-queue" --examples
        ;;
    esac

    cargo check --target $TARGET
    for ex in ${examples[@]}; do
        cargo check --target $TARGET --features cm7-r0p1 --example $ex
    done
    cargo check --features timer-queue --target $TARGET
    cargo check --features timer-queue --target $TARGET --examples
}

main
