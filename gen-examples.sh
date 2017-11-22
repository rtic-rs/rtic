# Converts the examples in the `examples` directory into documentation in the
# `examples` module (`src/examples/*.rs`)

set -ex

main() {
    local examples=(
        zero-tasks
        one-task
        two-tasks
        preemption
        nested
        late-resources
        generics
        full-syntax
    )

    rm -rf src/examples

    mkdir src/examples

    cat >src/examples/mod.rs <<'EOF'
//! Examples
// Auto-generated. Do not modify.
EOF

    local i=0 out=
    for ex in ${examples[@]}; do
        name=_${i}_${ex//-/_}
        out=src/examples/${name}.rs

        echo "pub mod $name;" >> src/examples/mod.rs

        grep '//!' examples/$ex.rs > $out
        echo '//!' >> $out
        echo '//! ```' >> $out
        grep -v '//!' examples/$ex.rs | (
            IFS=''

            while read line; do
                echo "//! $line" >> $out;
            done
        )
        echo '//! ```' >> $out
        echo '// Auto-generated. Do not modify.' >> $out


        chmod -x $out

        i=$(( i + 1 ))
    done

    chmod -x src/examples/mod.rs
}

main
