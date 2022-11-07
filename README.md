# rusty-mips

## Third-party dependency

Here are some major dependency:

This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).
It implies using Rust for making native Node.js plugin.

This project uses [Electron](https://www.electronjs.org/).

## Available Scripts

In the project directory, you can run:

### `npm install`

Installs the project, including running `npm run build`.

### `npm start`

Starts the Electron app in debug mode.
This automatically runs `npm run build-debug`.

### `npm run serve`

Starts the Electron app in release mode.
This automatically runs `npm run build-release`.

### `npm run build`

An alias for `npm run build-debug`, which builds the Rust component in debug mode.

### `npm run build-release`

Builds the Rust component in release mode.

### `npm run dist`

Packages the Electron app for current platform.
Due to complicated nature of building a native Rust library,
it is only possible to build for current platform only.

It *could* be possible to workaround above limitation by using Docker.

This runs `npm run build-release`.

