#!/bin/bash
set -e

./node_modules/.bin/ray build -e dist

# Raycast が起動中であれば develop を一時的に実行して拡張を登録する
if pgrep -x "Raycast" > /dev/null 2>&1; then
  ./node_modules/.bin/ray develop &
  RDEV_PID=$!
  sleep 5
  kill $RDEV_PID 2>/dev/null || true
  echo "✓ Registered with Raycast"
fi
