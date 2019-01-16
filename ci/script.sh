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
            ( cd book/en && mdbook build )
            ( cd book/ru && mdbook build )

            local td=$(mktemp -d)
            cp -r target/doc $td/api
            mkdir $td/book
            cp -r book/en/book $td/book/en
            cp -r book/ru/book $td/book/ru
            cp LICENSE-* $td/book/en
            cp LICENSE-* $td/book/ru

            linkchecker $td/book/en/
            linkchecker $td/book/ru/
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

                test_arm_example() {
                    local EXAMPLE=$1
                    local TARGET=$2
                    local BUILD_MODE=$3
                    local FEATURES=$4

                    if [ $BUILD_MODE = "release" ]; then
                        local RELEASE_FLAG="--release"
                    else
                        local RELEASE_FLAG=""
                    fi

                    if [ -n "$FEATURES" ]; then
                        local FEATURES_FLAG="--features $FEATURES"
                    else
                        local FEATURES_FLAG=""
                    fi
                    local CARGO_FLAGS="--example $EXAMPLE --target $TARGET $RELEASE_FLAG $FEATURES_FLAG"

                    cargo run $CARGO_FLAGS | diff -u ci/expected/$EXAMPLE.run -
                    arm-none-eabi-objcopy -O ihex target/$TARGET/$BUILD_MODE/examples/$EXAMPLE ${EXAMPLE}_1.hex

                    # build again to ensure that the build is reproducable
                    cargo clean
                    cargo build $CARGO_FLAGS
                    arm-none-eabi-objcopy -O ihex target/$TARGET/$BUILD_MODE/examples/$EXAMPLE ${EXAMPLE}_2.hex

                    # compare results of both builds
                    cmp ${EXAMPLE}_1.hex ${EXAMPLE}_2.hex
                }

                if [ $ex != types ]; then
                    test_arm_example $ex $T "debug" ""
                    test_arm_example $ex $T "release" ""
                fi

                if [ $TARGET != thumbv6m-none-eabi ]; then
                    test_arm_example $ex $T "debug" "timer-queue"
                    test_arm_example $ex $T "release" "timer-queue"
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

if [ -z ${TARGET-} ]; then
    TARGET=$(rustc -Vv | grep host | cut -d ' ' -f2)
fi

if [ $TRAVIS_BRANCH != master ] || [ $TRAVIS_PULL_REQUEST != false ]; then
    main
fi
