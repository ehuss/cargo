#!/bin/bash
# This script validates that there aren't any changes to the man pages.

set -e

for command in asciidoctor make man col git
do
    if ! command -v $command &> /dev/null
    then
        echo "$command not installed"
        exit 1
    fi
done

cd src/doc
# Force make to rebuild the docs, since it is timestamp based, but git does
# not preserve timestamps.
touch man/*.adoc

changes=$(git status --porcelain .)
if [ -n "$changes" ]
then
    echo "git directory must be clean before running this script:"
    echo "$changes"
    git diff
    exit 1
fi

make
changes=$(git status --porcelain .)
if [ -n "$changes" ]
then
    echo "Detected changes in man pages:"
    echo "$changes"
    echo
    git diff
    echo
    echo "Please run 'make' in the src/doc directory to rebuild the man pages."
    exit 1
fi
