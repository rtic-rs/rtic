#!/bin/bash

set -e

mkredirect() {
    mkdir -p $(dirname $2)
    sed -e "s|URL|$1|g" redirect.html > $2
}

langs=( en )
devver=( dev )
vers=( $1 )
webroot=${WEB_ROOT:-"book-target/deploy"}
oldbooks=${OLD_BOOKS_ROOT:-"book-target/old"}
current_book=${CURRENT_BOOK_ROOT:-"book-target/current"}

stable="${vers[0]}"
oldstable="${vers[1]}"

if [ -z "$CURRENT_VERSION" ]; then
    CURRENT_VERSION=$(./scripts/parse-version.sh)
fi

crate_version="$CURRENT_VERSION"

echo "Latest stable version: $stable"
echo "Current crate version: $crate_version"

# Create directories
rm -rf $webroot
mkdir -p $webroot/$devver

# Copy the current dev version
echo "Copy current dev version"
cp -r $current_book/* $webroot/$devver

echo "Inserting redirects"
# Replace relevant links to make rtic.rs/meeting/index.html
# redirect to the meeting and make the text a bit more descriptive
mkredirect "https://hackmd.io/c_mFUZL-Q2C6614MlrrxOg" $webroot/meeting/index.html
sed -e "s|Page Redirection|RTIC Meeting|g"              \
    -e "s|If you|Redirecting to RTIC HackMD. If you|g"  \
    -i $webroot/meeting/index.html

# Redirect the main site to the stable release
mkredirect "$stable" $webroot/index.html

# Create redirects for the dev version
if [ "$stable" != "$crate_version" ]; then
    # Current stable version being built differ
    # so we want to display the current dev version
    echo "Redirecting dev version dev version files"
    mkredirect "rtic/index.html" $webroot/$devver/api/index.html
    mkredirect "book/en" $webroot/$devver/index.html
else
    # The stable and crate version are the same
    # so we redirec to the stable version instead
    echo "Redirecting dev version to stable"
    mkredirect "https://rtic.rs/$stable/api/rtic" $webroot/$devver/api/index.html
    mkredirect "https://rtic.rs/$stable" $webroot/$devver/index.html
fi

# Pack up all of the older versions, including stable

echo "Copying stable"

# Copy the stable book to the stable alias
cp -r $oldbooks/$stable $webroot/stable

echo "Copying older versions"

# Copy the stable book to the webroot
cp -r $oldbooks/$stable $webroot/
# Copy the old stable book to the webroot
cp -r $oldbooks/$oldstable $webroot/

# Forward CNAME file
cp CNAME $webroot
