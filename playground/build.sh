#!/bin/bash
set -e

# Build the WASM package
echo "Building WASM package..."
cd "$(dirname "$0")"
wasm-pack build --target web --out-dir www/pkg

# Clean up unnecessary files
rm -f www/pkg/.gitignore www/pkg/package.json www/pkg/README.md

echo "Build complete! Files are in playground/www/"
echo ""
echo "To test locally, run:"
echo "  cd playground/www && python3 -m http.server 8080"
echo ""
echo "Then open http://localhost:8080 in your browser."
