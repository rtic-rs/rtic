set -euxo pipefail

arm_example() {
    local COMMAND=$1
    local EXAMPLE=$2
    local BUILD_MODE=$3
    local FEATURES=$4
    local BUILD_NUM=$5

    if [ $BUILD_MODE = "release" ]; then
        local RELEASE_FLAG="--release"
    else
        local RELEASE_FLAG=""
    fi

    if [ -n "$FEATURES" ]; then
        local FEATURES_FLAG="--features $FEATURES"
        local FEATURES_STR=${FEATURES/,/_}_
    else
        local FEATURES_FLAG=""
        local FEATURES_STR=""
    fi
    local CARGO_FLAGS="--example $EXAMPLE --target $TARGET $RELEASE_FLAG $FEATURES_FLAG"

    if [ $COMMAND = "run" ]; then
        cargo $COMMAND $CARGO_FLAGS | diff -u ci/expected/$EXAMPLE.run -
    else
        cargo $COMMAND $CARGO_FLAGS
    fi
    arm-none-eabi-objcopy -O ihex target/$TARGET/$BUILD_MODE/examples/$EXAMPLE ${EXAMPLE}_${FEATURES_STR}${BUILD_MODE}_${BUILD_NUM}.hex
}


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
                    arm_example "run" $ex "debug" "" "1"
                    arm_example "run" $ex "release" "" "1"
                fi

                if [ $TARGET != thumbv6m-none-eabi ]; then
                    arm_example "run" $ex "debug" "timer-queue" "1"
                    arm_example "run" $ex "release" "timer-queue" "1"
                fi
            done

            cargo clean
            for ex in ${exs[@]}; do
                if [ $ex != types ]; then
                    arm_example "build" $ex "debug" "" "2"
                    cmp ${ex}_debug_1.hex ${ex}_debug_2.hex
                    arm_example "build" $ex "release" "" "2"
                    cmp ${ex}_release_1.hex ${ex}_release_2.hex
                fi

                if [ $TARGET != thumbv6m-none-eabi ]; then
                    arm_example "build" $ex "debug" "timer-queue" "2"
                    cmp ${ex}_timer-queue_debug_1.hex ${ex}_timer-queue_debug_2.hex
                    arm_example "build" $ex "release" "timer-queue" "2"
                    cmp ${ex}_timer-queue_release_1.hex ${ex}_timer-queue_release_2.hex
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
