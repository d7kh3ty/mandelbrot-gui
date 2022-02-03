OUTPUT=static/public
TARGET=wasm32-unknown-emscripten

all:
	EMCC_CFLAGS="-s USE_SDL=2 -s ERROR_ON_UNDEFINED_SYMBOLS=0 --no-entry" cargo build --release --target $(TARGET)
	mkdir -p $(OUTPUT)
	find target/wasm32-unknown-emscripten/release/deps -type f -name "*.wasm" | xargs -I {} mv {} $(OUTPUT)/
	find target/wasm32-unknown-emscripten/release/deps -type f ! -name "*.asm.js" -name "*.js" | xargs -I {} mv {} $(OUTPUT)/
	find target/wasm32-unknown-emscripten/release/deps -type f -name "*.data" | xargs -I {} mv {} $(OUTPUT)/

clean:
	rm -rf target static/public/*.wasm static/public/*.d static/public/*.rlib static/public/*.js

serve:
	cd static && npm run serve
