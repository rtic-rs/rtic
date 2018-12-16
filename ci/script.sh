set -euxo pipefail

main() {
    local T=$TARGET

    if [ $T = x86_64-unknown-linux-gnu ]; then
        # compile-fail and compile-pass tests
        case $TRAVIS_RUST_VERSION in
            nightly*)
                # TODO how to run a subset of these tests when timer-queue is disabled?
                cargo test --features timer-queue --test compiletest --target $T
        esac

        cargo check --target $T
        if [ $TARGET != thumbv6m-none-eabi ]; then
            cargo check --features timer-queue --target $T
        fi

        if [ $TRAVIS_RUST_VERSION != nightly ]; then
            rm -f .cargo/config
            if [ $TARGET != thumbv6m-none-eabi ]; then
                cargo doc --features timer-queue
            else
                cargo doc
            fi
            ( cd book && mdbook build )

            local td=$(mktemp -d)
            cp -r target/doc $td/api
            cp -r book/book $td/
            cp LICENSE-* $td/book/

            linkchecker $td/book/
            linkchecker $td/api/rtfm/
            linkchecker $td/api/cortex_m_rtfm_macros/
        fi

        return
    fi

    cargo check --target $T --examples
    if [ $TARGET != thumbv6m-none-eabi ]; then
        cargo check --features timer-queue --target $T --examples
    fi

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

                generics
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

                if [ $TARGET != thumbv6m-none-eabi ]; then
                    cargo run --features timer-queue --example $ex --target $T | \
                        diff -u ci/expected/$ex.run -

                    cargo run --features timer-queue --example $ex --target $T --release | \
                        diff -u ci/expected/$ex.run -
                fi
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

if [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_PULL_REQUEST != false ]; then
    main
fi
