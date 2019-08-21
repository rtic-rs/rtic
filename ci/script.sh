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
        if [ $TRAVIS_RUST_VERSION = nightly ]; then
            # compile-fail tests
            cargo test --test single --target $T

            # multi-core compile-pass tests
            pushd heterogeneous
            local exs=(
                smallest
                x-init-2
                x-init
                x-schedule
                x-spawn
            )
            for ex in ${exs[@]}; do
                cargo microamp --example $ex --target thumbv7m-none-eabi,thumbv6m-none-eabi --check
            done

            popd

        else
            if [ $TRAVIS_RUST_VERSION != nightly ]; then
                rm -f .cargo/config
                cargo doc
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
        fi

        cargo check --target $T
        ( cd macros && cargo test --target $T )

        return
    fi

    if [ $TARGET = thumbv6m-none-eabi ]; then
        cargo check --target $T --examples
    else
        cargo check --target $T --examples --features __v7
    fi

    cargo check -p homogeneous --target $T --examples

    # run-pass tests
    case $T in
        thumbv6m-none-eabi | thumbv7m-none-eabi)
            local exs=(
                idle
                init
                hardware
                preempt
                binds

                resource
                lock
                late
                only-shared-access

                task
                message
                capacity

                types
                not-send
                not-sync
                shared-with-init

                generics
                cfg
                pool
                ramfunc
            )

            for ex in ${exs[@]}; do
                if [ $ex = pool ]; then
                    if [ $TARGET = thumbv6m-none-eabi ]; then
                        continue
                    fi

                    local td=$(mktemp -d)

                    cargo run --example $ex --target $TARGET --features __v7 >\
                            $td/pool.run
                    grep 'foo(0x2' $td/pool.run
                    grep 'bar(0x2' $td/pool.run
                    arm-none-eabi-objcopy -O ihex target/$TARGET/debug/examples/$ex \
                                            ci/builds/${ex}___v7_debug_1.hex

                    cargo run --example $ex --target $TARGET --features __v7 --release >\
                            $td/pool.run
                    grep 'foo(0x2' $td/pool.run
                    grep 'bar(0x2' $td/pool.run
                    arm-none-eabi-objcopy -O ihex target/$TARGET/release/examples/$ex \
                                            ci/builds/${ex}___v7_release_1.hex

                    rm -rf $td

                    continue
                fi

                if [ $ex = types ]; then
                    if [ $TARGET = thumbv6m-none-eabi ]; then
                        continue
                    fi

                    arm_example "run" $ex "debug" "__v7" "1"
                    arm_example "run" $ex "release" "__v7" "1"

                    continue
                fi

                arm_example "run" $ex "debug" "" "1"
                if [ $ex = types ]; then
                    arm_example "run" $ex "release" "" "1"
                else
                    arm_example "build" $ex "release" "" "1"
                fi
            done

            local built=()
            cargo clean
            for ex in ${exs[@]}; do
                if [ $ex = types ] || [ $ex = pool ]; then
                    if [ $TARGET = thumbv6m-none-eabi ]; then
                        continue
                    fi

                    arm_example "build" $ex "debug" "__v7" "2"
                    cmp ci/builds/${ex}___v7_debug_1.hex \
                        ci/builds/${ex}___v7_debug_2.hex
                    arm_example "build" $ex "release" "__v7" "2"
                    cmp ci/builds/${ex}___v7_release_1.hex \
                        ci/builds/${ex}___v7_release_2.hex
                else
                    arm_example "build" $ex "debug" "" "2"
                    cmp ci/builds/${ex}_debug_1.hex \
                        ci/builds/${ex}_debug_2.hex
                    arm_example "build" $ex "release" "" "2"
                    cmp ci/builds/${ex}_release_1.hex \
                        ci/builds/${ex}_release_2.hex
                fi

                built+=( $ex )
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
