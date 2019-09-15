set -euxo pipefail

main() {
    local langs=( en ru )
    local vers=( 0.4.3 )

    rm -f .cargo/config
    cargo doc

    local td=$(mktemp -d)

    # build latest docs
    cp -r target/doc $td/api
    sed 's|URL|rtfm/index.html|g' redirect.html > $td/api/index.html

    mkdir $td/book/
    sed 's|URL|book/en|g' redirect.html > $td/index.html
    for lang in ${langs[@]}; do
        ( cd book/$lang && mdbook build )
        cp -r book/$lang/book $td/book/$lang
        cp LICENSE-* $td/book/$lang/
    done

    local root=$(pwd)
    # build older docs
    for ver in ${vers[@]}; do
        local prefix=${ver%.*}

        mkdir -p $td/$prefix/book
        local src=$(mktemp -d)
        curl -L https://github.com/rtfm-rs/cortex-m-rtfm/archive/v${ver}.tar.gz | tar xz --strip-components 1 -C $src

        pushd $src
        rm -f .cargo/config
        cargo doc || cargo doc --features timer-queue
        cp -r target/doc $td/$prefix/api
        sed 's|URL|rtfm/index.html|g' $root/redirect.html > $td/$prefix/api/index.html
        for lang in ${langs[@]}; do
            ( cd book/$lang && mdbook build )
            cp -r book/$lang/book $td/$prefix/book/$lang
            cp LICENSE-* $td/$prefix/book/$lang/
        done
        sed 's|URL|book/en|g' $root/redirect.html > $td/$prefix/index.html
        popd

        rm -rf $src
    done

    # forward CNAME file
    cp CNAME $td/

    mkdir ghp-import
    curl -Ls https://github.com/davisp/ghp-import/archive/master.tar.gz |
        tar --strip-components 1 -C ghp-import -xz

    ./ghp-import/ghp_import.py $td

    set +x
    git push -fq https://$GH_TOKEN@github.com/rtfm-rs/cortex-m-rtfm.git gh-pages && echo OK

    rm -rf $td
}

# fake Travis variables to be able to run this on a local machine
if [ -z ${TRAVIS_BRANCH-} ]; then
    TRAVIS_BRANCH=master
fi

if [ -z ${TRAVIS_PULL_REQUEST-} ]; then
    TRAVIS_PULL_REQUEST=false
fi

if [ $TRAVIS_BRANCH = master ] && [ $TRAVIS_PULL_REQUEST = false ]; then
    main
fi
