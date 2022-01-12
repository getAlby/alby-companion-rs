# Alby Companion app

The Alby companion app allows Alby to connect to nodes that run behind Tor or are otherwise not easily accessible.

# Build
In the project folder, run:  
```
cargo build --release
```
after this you'll get an executable in `./target/release/alby`

# Debug

Temporary directory might be generated in a folder with random (unique) name, so for debug you might want to run the app with "--debug" flag:  
```
cargo run -- --debug
```
In this mode you'll get the log file name.  
When running as a native companion app check the log file and if the process is running.

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


# Setup Notes

## MacOS
You'll need [Homebrew](https://brew.sh/) and [installed Rust](https://rustup.rs/).
  
1. OpenSSL
```
brew install openssl@1.1 
```
then
```
echo 'export PATH="/opt/homebrew/opt/openssl@1.1/bin:$PATH"' >> ~/.zshrc
echo 'export LDFLAGS="-L/opt/homebrew/opt/openssl@1.1/lib"' >> ~/.zshrc 
echo 'export CPPFLAGS="-I/opt/homebrew/opt/openssl@1.1/include"' >> ~/.zshrc
```
2. Compilation tools
```
 brew install gcc libevent autoconf automake    
```
After that restart your terminal.


## Linux

### Ubuntu Dependencies

* build-essential, autoconf, libssl-dev
* [rustup](https://rustup.rs/)


# Install the companion app for Alby

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

