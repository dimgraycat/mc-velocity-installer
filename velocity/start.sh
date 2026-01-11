#!/usr/bin/env sh
set -e
DIR="$(cd "$(dirname "$0")" && pwd)"
exec java -Xms256M -Xmx512M -jar "$DIR/velocity.jar"
