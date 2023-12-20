#!/bin/bash
IFS='.' read -r major minor patch < VERSION
echo "${major}.$((minor + 1)).0" > VERSION