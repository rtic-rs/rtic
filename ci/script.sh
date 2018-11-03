set -euxo pipefail

main() {
    local T=$TARGET

    if [ $T = x86_64-unknown-linux-gnu ]; then
        # compile-fail and compile-pass tests
        if [ $TRAVIS_RUST_VERSION = nightly ]; then
            # TODO how to run a subset of these tests when timer-queue is disabled?
            cargo test --features timer-queue --test compiletest --target $T
        fi

        cargo check --target $T
        cargo check --features timer-queue --target $T
        return
    fi

    cargo check --target $T --examples
    cargo check --features timer-queue --target $T --examples

    # run-pass tests
    case $T in
        thumbv6m-none-eabi | thumbv7m-none-eabi)
            local exs=(
                idle
                init
                interrupt

                resource
                lock
                late
                static

                task
                message
                capacity

                singleton

                types
                not-send
                not-sync

                ramfunc
            )

            for ex in ${exs[@]}; do
                if [ $ex = ramfunc ] && [ $T = thumbv6m-none-eabi ]; then
                    # LLD doesn't support this at the moment
                    continue
                fi

                if [ $ex != types ]; then
                    cargo run --example $ex --target $T | \
                        diff -u ci/expected/$ex.run -

                    cargo run --example $ex --target $T --release | \
                        diff -u ci/expected/$ex.run -
                fi

                cargo run --features timer-queue --example $ex --target $T | \
                    diff -u ci/expected/$ex.run -

                cargo run --features timer-queue --example $ex --target $T --release | \
                    diff -u ci/expected/$ex.run -
            done
    esac
}

# fake Travis variables to be able to run this on a local machine
if [ -z ${TRAVIS_BRANCH-} ]; then
    TRAVIS_BRANCH=auto
fi

if [ -z ${TRAVIS_PULL_REQUEST-} ]; then
    TRAVIS_PULL_REQUEST=false
fi

if [ -z ${TRAVIS_RUST_VERSION-} ]; then
    case $(rustc -V) in
        *nightly*)
            TRAVIS_RUST_VERSION=nightly
            ;;
        *beta*)
            TRAVIS_RUST_VERSION=beta
            ;;
        *)
            TRAVIS_RUST_VERSION=stable
            ;;
    esac
fi

if [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_PULL_REQUEST = true ]; then
    main
fi
