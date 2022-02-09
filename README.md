# requirements

```
sdl2
npm
```

# instructions for use

## running on the desktop

`cargo run`

## in the browser

```
cd emsdk

# Fetch the latest version of the emsdk (not needed the first time you clone)
git pull

# Download and install the latest SDK tools.
./emsdk install latest

# Make the "latest" SDK "active" for the current user. (writes .emscripten file)
./emsdk activate latest

# Activate PATH and other environment variables in the current terminal
source ./emsdk_env.sh

cd ../static

# Install required node modules
npm install

# Run the web server
npm run serve
```
