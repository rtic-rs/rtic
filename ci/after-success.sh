set -euxo pipefail

main() {
    rm -f .cargo/config
    cargo doc --features timer-queue
    ( cd book && mdbook build )

    local td=$(mktemp -d)
    cp -r target/doc $td/api
    cp -r book/book $td/
    cp LICENSE-* $td/book/

    mkdir ghp-import
    curl -Ls https://github.com/davisp/ghp-import/archive/master.tar.gz |
        tar --strip-components 1 -C ghp-import -xz

    ./ghp-import/ghp_import.py $td

    set +x
    git push -fq https://$GH_TOKEN@github.com/$TRAVIS_REPO_SLUG.git gh-pages && echo OK

    rm -rf $td
}

if [ $TRAVIS_BRANCH = master ] && [ $TRAVIS_PULL_REQUEST = false ]; then
    main
fi
