#!/bin/sh
set -eu

REPO="flomotion-app/flomotion-desktop"
DIR="${FLOMOTION_HOME:-$HOME/.flomotion}/bin"

os=$(uname -s)
arch=$(uname -m)
case "$os" in
  Linux) archive="flomotion-linux-x86_64.tar.gz" ;;
  Darwin)
    case "$arch" in
      arm64) archive="flomotion-macos-arm64.tar.gz" ;;
      *) archive="flomotion-macos-x86_64.tar.gz" ;;
    esac ;;
  *) echo "unsupported OS: $os"; exit 1 ;;
esac

url="https://github.com/$REPO/releases/latest/download/$archive"
tmp=$(mktemp -d)
echo "downloading $url"
curl -fsSL "$url" -o "$tmp/$archive"
mkdir -p "$DIR"
tar -xzf "$tmp/$archive" -C "$tmp"
cp "$tmp/flomotion/flomotion" "$DIR/flomotion"
chmod +x "$DIR/flomotion"
rm -rf "$tmp"

echo "installed $DIR/flomotion"
case ":$PATH:" in
  *":$DIR:"*) ;;
  *) echo "add it to PATH: export PATH=\"$DIR:\$PATH\"" ;;
esac
if [ "$os" = "Linux" ] && ! ldconfig -p 2>/dev/null | grep -q libwebkit2gtk-4.1; then
  echo "WebKitGTK is required: sudo apt-get install libwebkit2gtk-4.1-0 (Debian/Ubuntu)"
fi
echo "next: flomotion skill"
