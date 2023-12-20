#!/bin/bash
IFS='.' read -r major minor patch < VERSION
echo "$((major + 1)).0.0" > VERSION