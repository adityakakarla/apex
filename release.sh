#!/bin/zsh
set -e  # stop on any error

rm apex.dmg

cargo bundle --release

create-dmg \
  --volname "Apex" \
  --window-size 600 400 \
  --icon-size 100 \
  --icon "apex.app" 150 200 \
  --hide-extension "apex.app" \
  --app-drop-link 450 200 \
  "apex.dmg" \
  "target/release/bundle/osx/"

echo "Done! apex.dmg is ready."
