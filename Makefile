.PHONY: tag_release push_release release major minor patch

# Project settings
PROJECT_NAME := sonomar

# Function to read version from file
get_version = $(shell cat VERSION)

# Tag the release
tag_release:
	@git tag -a $(call get_version) -m "Release $(call get_version)"
	@git push origin $(call get_version)

# Push the release to GitHub
push_release:
	@git push origin --tags

# Full release process
release: tag_release push_release

# Major version bump
major:
	@./bump_major.sh
	@git commit -am "Bump version to $(call get_version)"
	@git push
	@echo "Major version bumped to $(call get_version)"

# Minor version bump
minor:
	@./bump_minor.sh
	@git commit -am "Bump version to $(call get_version)"
	@git push
	@echo "Minor version bumped to $(call get_version)"

# Patch version bump
patch:
	@./bump_patch.sh
	@git commit -am "Bump version to $(call get_version)"
	@git push
	@echo "Patch version bumped to $(call get_version)"
