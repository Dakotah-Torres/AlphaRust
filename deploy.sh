#!/bin/bash
# 1. Commit and push current dev work
git add .
git commit -m "$1"
git push origin dev

# 2. Merge into main
git checkout main
git merge dev -m "Merge dev: $1"
git push origin main

# 3. Back to dev
git checkout dev