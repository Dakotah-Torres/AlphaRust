#!/bin/bash
# 1. Commit and push current dev work
git add .
git commit -m "$1"
git push origin dev

# 2. Merge into main
git checkout master
git merge dev -m "Merge dev: $1"
git push origin master

# 3. Back to dev
git checkout dev