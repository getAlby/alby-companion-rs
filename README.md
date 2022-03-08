# Alby Companion app

The Alby companion app allows [Alby](https://getalby.com/) to connect to nodes that run behind Tor or are otherwise not easily accessible on the public clearnet internet.  
It serves as native companion app for the [Alby lightning browser extension](https://getalby.com/). The browser extension uses the [browser's native messaging](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging) to talk to this native app.

**NOTE:** Typically the user does not manually install this app but uses an app package like for [macOS](https://github.com/getAlby/alby-installer-macos) or [Windows](https://github.com/getAlby/alby-installer-windows)


# Build
In the project folder, run:  
```
cargo build --release
```

**OSX:** 
```
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo target/aarch64-apple-darwin/release/alby target/x86_64-apple-darwin/release/alby -create -output alby
```
with this you'll get an universal executable in `./alby`
You can use the `./build-macos.sh` script to run the build and create a release zip file.

# Debug

Temporary directory might be generated in a folder with random (unique) name, so for debug you might want to run the app with "--debug" flag:  
```
cargo run -- --debug
```
In this mode you'll get the log file name.  

When running as a native companion app check the log file and if the process is running.
```
tail -f $TMPDIR/alby.log
```

# Command-line options

* `--log-file`, `--log_file`, `--l` - string;     
* `--tor-dir`, `--tor_dir`, `--t` - string;  
* `--debug` - presence of this flag will turn on the debug mode.

## Run with option

Executable:  

```
/some/folder/alby --log-file=/tmp/alby.log
```

Debug:  

```
cargo run -- --log-file=/tmp/alby.log
```


# Development Setup Notes

## MacOS
You'll need [Homebrew](https://brew.sh/) and [installed Rust](https://rustup.rs/).

```
brew install openssl gcc libevent autoconf automake    
```


## Linux

### Ubuntu Dependencies

* build-essential, autoconf, libssl-dev
* [rustup](https://rustup.rs/)


# Install the companion app for Alby

**NOTE:** Typically the user does not manually install this app but uses an app package like for [macOS](https://github.com/getAlby/alby-installer-macos) or [Windows](https://github.com/getAlby/alby-installer-windows)


## Firefox

1. build the companion app (`cargo build --release`)
2. copy the `manifest-firefox.json` example to the Firefox NativeMessagingHosts folder of your system (see below)
3. edit the path in that `alby.json` file and profile the full absolute path to your `alby` executable.

More information: [Mozilla docs](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging
)

#### Folder for the manifest file

* **OSX:** `~/Library/Application Support/Mozilla/NativeMessagingHosts/alby.json`
* **Linux:** `~/.mozilla/native-messaging-hosts/alby.json`
* **Windows:** Create a Registry entry `HKEY_CURRENT_USER\SOFTWARE\Mozilla\NativeMessagingHosts\alby` with the default value as path to the manifest json file

More details: [WebExtensions/Native_manifests](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests)


## Chrome

1. build the companion app (`cargo build --release`)
2. copy the `manifest-chome.json` example to the Chrome NativeMessagingHosts folder of your system (see below)
3. edit the path in that `alby.json` file and profile the full absolute path to your `alby` executable
4. make sure the extension ID is correct in that `alby.json` file (it must end with a `/`)

More information: [Chome docs](https://developer.chrome.com/docs/apps/nativeMessaging/)

#### Folder for the manifest file

* **OSX:** `~/Library/Application Support/Google/Chrome/NativeMessagingHosts/alby.json`
* **Linux:** `~/.config/google-chrome/NativeMessagingHosts/alby.json`
* **Windows:** [see here](https://developer.chrome.com/docs/apps/nativeMessaging/#native-messaging-host-location)

More details: [WebExtensions/Native_manifests](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_manifests)

