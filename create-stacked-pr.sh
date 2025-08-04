#!/usr/bin/env bash
set -e

revision="$1"
if [ -z "$revision" ]; then
    echo "Usage: $0 <revision>"
    exit 1
fi

jj git push --change $revision

revBookmark=$(jj log -r $revision --no-graph -T bookmarks)
revDescription=$(jj log -r $revision --no-graph -T description)
parentBookmark=$(jj log -r "latest(bookmarks(push) & ancestors(parents($revision)))" --no-graph -T bookmarks)

echo "revBookmark: $revBookmark"
echo "parentBookmark: $parentBookmark"

gh pr create --base $parentBookmark --head $revBookmark --title "$revDescription" --body "" --assignee "@me"
