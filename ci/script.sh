set -euxo pipefail

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo build --target $TARGET
        cargo test --test cfail --target $TARGET
        return
    fi

    # examples that don't require the timer-queue feature
    local examples=(
        schedule-now
        empty
        event-task
    )

    # without timer-queue
    cargo check --target $TARGET

    for ex in ${examples[@]}; do
        cargo build --target $TARGET --example $ex
        cargo build --target $TARGET --example $ex --release
    done

    # with timer-queue
    cargo check --features timer-queue --target $TARGET

    cargo build --features timer-queue --target $TARGET --examples
    cargo build --features timer-queue --target $TARGET --examples --release

    # test again but with the cm7-r0p1 feature enabled
    case $TARGET in
        thumbv7em-none-eabi*)
            # without timer-queue
            cargo check --target $TARGET --features cm7-r0p1

            for ex in ${examples[@]}; do
                cargo build --target $TARGET --features cm7-r0p1 --example $ex
                cargo build --target $TARGET --features cm7-r0p1 --example $ex --release
            done

            # with timer-queue
            cargo check --target $TARGET --features "cm7-r0p1 timer-queue"

            cargo build --target $TARGET --features "cm7-r0p1 timer-queue" --examples
            cargo build --target $TARGET --features "cm7-r0p1 timer-queue" --examples --release
            ;;
    esac
}

main
