# rusty-mips

A MIPS simulator written in Rust.

## Third-party dependency

This project was bootstrapped by [create-neon](https://www.npmjs.com/package/create-neon).
It implies using Rust for making native Node.js plugin.

This project uses [Electron](https://www.electronjs.org/).

## Available Scripts

In the project directory, you can run:

### `npm install`

Installs the project.

### `npm start`

Starts the Electron app in debug mode.
This builds the Rust module in debug mode, and React also uses debug mode.

### `npm run serve`

Starts the Electron app in release mode.
This builds the Rust module in release mode, while React uses debug mode.

### `npm run dist`

Packages the Electron app for current platform.
Due to complicated nature of building a native library,
it is only possible to build for current platform only.

It *could* be possible to workaround above limitation by using Docker.

This builds the Rust module in release mode.

