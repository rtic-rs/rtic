set -euxo pipefail

main() {
    local langs=( en ru )

    rm -f .cargo/config
    cargo doc --features timer-queue

    local td=$(mktemp -d)
    cp -r target/doc $td/api
    mkdir $td/book/
    cp redirect.html $td/book/index.html
    for lang in ${langs[@]}; do
        ( cd book/$lang && mdbook build )
        cp -r book/$lang/book $td/book/$lang
        cp LICENSE-* $td/book/$lang/
    done

    mkdir ghp-import
    curl -Ls https://github.com/davisp/ghp-import/archive/master.tar.gz |
        tar --strip-components 1 -C ghp-import -xz

    ./ghp-import/ghp_import.py $td

    set +x
    git push -fq https://$GH_TOKEN@github.com/$TRAVIS_REPO_SLUG.git gh-pages && echo OK

    rm -rf $td
}

# fake Travis variables to be able to run this on a local machine
if [ -z ${TRAVIS_BRANCH-} ]; then
    TRAVIS_BRANCH=master
fi

if [ -z ${TRAVIS_PULL_REQUEST-} ]; then
    TRAVIS_PULL_REQUEST=false
fi

if { [ $TRAVIS_BRANCH = master ] || [ $TRAVIS_BRANCH = v0.4.x ]; } && [ $TRAVIS_PULL_REQUEST = false ]; then
    main
fi
