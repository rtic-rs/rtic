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
    arm-none-eabi-objcopy -O ihex target/$TARGET/$BUILD_MODE/examples/$EXAMPLE ci/builds/${EXAMPLE}_${FEATURES_STR}${BUILD_MODE}_${BUILD_NUM}.hex
}


main() {
    local T=$TARGET

    mkdir -p ci/builds

    if [ $T = x86_64-unknown-linux-gnu ]; then
        # compile-fail and compile-pass tests

        # TODO how to run a subset of these tests when timer-queue is disabled?
        cargo test --features "timer-queue" --test compiletest --target $T

        cargo check --target $T
        if [ $TARGET != thumbv6m-none-eabi ]; then
            cargo check --features "timer-queue" --target $T
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
        cargo check --features "timer-queue" --target $T --examples
    fi

    # run-pass tests
    case $T in
        thumbv6m-none-eabi | thumbv7m-none-eabi)
            local exs=(
                idle
                init
                interrupt
                binds

                resource
                lock
                late
                static

                task
                message
                capacity

                types
                not-send
                not-sync
                shared-with-init

                generics
                pool
                ramfunc
            )

            for ex in ${exs[@]}; do
                if [ $ex = ramfunc ] && [ $T = thumbv6m-none-eabi ]; then
                    # LLD doesn't support this at the moment
                    continue
                fi

                if [ $ex = pool ]; then
                    if [ $TARGET != thumbv6m-none-eabi ]; then
                        local td=$(mktemp -d)

                        local features="timer-queue"
                        cargo run --example $ex --target $TARGET --features $features >\
                              $td/pool.run
                        grep 'foo(0x2' $td/pool.run
                        grep 'bar(0x2' $td/pool.run
                        arm-none-eabi-objcopy -O ihex target/$TARGET/debug/examples/$ex \
                                              ci/builds/${ex}_${features/,/_}_debug_1.hex

                        cargo run --example $ex --target $TARGET --features $features --release >\
                              $td/pool.run
                        grep 'foo(0x2' $td/pool.run
                        grep 'bar(0x2' $td/pool.run
                        arm-none-eabi-objcopy -O ihex target/$TARGET/release/examples/$ex \
                                              ci/builds/${ex}_${features/,/_}_release_1.hex

                        rm -rf $td
                    fi

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

            local built=()
            cargo clean
            for ex in ${exs[@]}; do
                if [ $ex = ramfunc ] && [ $T = thumbv6m-none-eabi ]; then
                    # LLD doesn't support this at the moment
                    continue
                fi

                if [ $ex != types ] && [ $ex != pool ]; then
                    arm_example "build" $ex "debug" "" "2"
                    cmp ci/builds/${ex}_debug_1.hex \
                        ci/builds/${ex}_debug_2.hex
                    arm_example "build" $ex "release" "" "2"
                    cmp ci/builds/${ex}_release_1.hex \
                        ci/builds/${ex}_release_2.hex

                    built+=( $ex )
                fi

                if [ $TARGET != thumbv6m-none-eabi ]; then
                    arm_example "build" $ex "debug" "timer-queue" "2"
                    cmp ci/builds/${ex}_timer-queue_debug_1.hex \
                        ci/builds/${ex}_timer-queue_debug_2.hex
                    arm_example "build" $ex "release" "timer-queue" "2"
                    cmp ci/builds/${ex}_timer-queue_release_1.hex \
                        ci/builds/${ex}_timer-queue_release_2.hex
                fi
            done

            ( cd target/$TARGET/release/examples/ && size ${built[@]} )
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
