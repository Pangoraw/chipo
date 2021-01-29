#!/bin/sh

cargo install wasm-pack
wasm-pack build --target web chipo-web

mkdir public

cp -r chipo-web/index.html chipo-web/pkg/ chipo-web/static/ public/

echo "Done building! 🛠️"
