set -ex

main() {
    if [ $TARGET = x86_64-unknown-linux-gnu ]; then
        cargo build
        cargo test
        return
    fi

    xargo build --target $TARGET
    for ex in $(ls examples/*); do
        ex=$(basename $ex)
        ex=${ex%.*}
        xargo build --target $TARGET --example $ex
    done
}

main
