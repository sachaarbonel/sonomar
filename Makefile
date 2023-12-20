.PHONY: tag_release push_release release

# Project settings
PROJECT_NAME := sonomar
VERSION := $(shell cat VERSION)

# Tag the release
tag_release:
    git tag -a $(VERSION) -m "Release $(VERSION)"
    git push origin $(VERSION)

# Push the release to GitHub
push_release:
    git push origin --tags

# Full release process
release: tag_release push_release
