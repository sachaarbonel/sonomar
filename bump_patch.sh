#!/bin/bash
IFS='.' read -r major minor patch < VERSION
echo "${major}.${minor}.$((patch + 1))" > VERSION