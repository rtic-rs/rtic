#!/bin/bash

set -e

mkredirect(){
    mkdir -p $(dirname $2)
    sed -e "s|URL|$1|g" $root/redirect.html > $2
}

vers=($1)
buildroot=${OLD_BOOK_BUILD_ROOT:-"book-target/old"}
webroot=$buildroot/output
rm -rf $webroot
mkdir -p $webroot
webroot=$(realpath $webroot)
root=$(pwd)

for ver in ${vers[@]}; do
    echo "Building book v${ver}"
    mkdir -p $buildroot/src/$ver
    src=$buildroot/src/$ver
    curl -fL https://github.com/rtic-rs/rtic/archive/release/v${ver}.tar.gz | tar xz --strip-components 1 -C $src

    pushd $src
    rm -rf .cargo/config

    # Build the docs: there are a few combinations we have to try to cover all of
    # the versions
    cargo doc || cargo doc --features thumbv7-backend

    mkdir -p $webroot/$ver/api
    cp -r $(realpath target/doc) $webroot/$ver/api

    mkredirect "rtic/index.html" $webroot/$ver/api/index.html

    # Build and copy all of the languages
    langs=( book/* )
    for lang in ${langs[@]}; do
        lang=$(basename $lang)
        lang_root=$webroot/$ver/book/$lang
        mkdir -p $lang_root
        pushd book/$lang
        echo $(pwd)
        mdbook build -d $lang_root
        popd
        cp LICENSE-* $lang_root
    done

    mkredirect "book/en" $webroot/$ver/index.html

    popd
    rm -r $src
done