#!/bin/sh

set -ex

env
now=`date '+%Y-%m-%d'`
ls -al
git config user.name "Cron Squash"
git config user.email ""
git remote -v show
# git push -f origin $GITHUB_SHA:refs/heads/snapshot-$now

# msg=$(cat <<-END
# Collapse index into one commit

# Previous HEAD was $GITHUB_SHA, now on the \`snapshot-$now\` branch

# More information about this change can be found [online] and on [this issue]

# [online]: https://internals.rust-lang.org/t/cargos-crate-index-upcoming-squash-into-one-commit/8440
# [this issue]: https://github.com/rust-lang/crates-io-cargo-teams/issues/47
# END
# )

# new_rev=$(git commit-tree HEAD^{tree} -m "$msg")

# git push origin $new_rev:refs/heads/master \
#   --force-with-lease=refs/heads/master:$GITHUB_SHA
