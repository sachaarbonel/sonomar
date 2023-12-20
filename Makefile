.PHONY: tag_release push_release release major minor patch

# Project settings
PROJECT_NAME := sonomar
VERSION := $(shell cat VERSION)

# Function to bump version numbers
bump_version = $(shell echo $(1) | awk -F. -v OFS=. '{$(2)++; for (i=$(3); i<NF; i++) $i = 0; print}')

# Tag the release
tag_release:
	@git tag -a $(VERSION) -m "Release $(VERSION)"
	@git push origin $(VERSION)

# Push the release to GitHub
push_release:
	@git push origin --tags

# Full release process
release: tag_release push_release

# Major version bump
major:
	@$(eval NEW_VERSION := $(call bump_version,0.$(VERSION)))
	@echo $(NEW_VERSION) > VERSION
	@git commit -am "Bump version to $(NEW_VERSION)"
	@git push
	@echo "Major version bumped to $(NEW_VERSION)"

# Minor version bump
minor:
	@$(eval NEW_VERSION := $(call bump_version,1.$(VERSION)))
	@echo $(NEW_VERSION) > VERSION
	@git commit -am "Bump version to $(NEW_VERSION)"
	@git push
	@echo "Minor version bumped to $(NEW_VERSION)"

# Patch version bump
patch:
	@$(eval NEW_VERSION := $(call bump_version,2.$(VERSION)))
	@echo $(NEW_VERSION) > VERSION
	@git commit -am "Bump version to $(NEW_VERSION)"
	@git push
	@echo "Patch version bumped to $(NEW_VERSION)"
