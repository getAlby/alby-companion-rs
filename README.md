# Setup
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

# Install companion app

https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging

* Edit `manifest-firefox.json`  
* Copy `manifest-firefox.json` to `~/Library/Application Support/Mozilla/NativeMessagingHosts/<name>.json`

    $ cp manifest-firefox.json "~/Library/Application Support/Mozilla/NativeMessagingHosts/alby.json"

If there's no such folder:  
```
mkdir -p "~/Library/Application Support/Mozilla/NativeMessagingHosts/"
```
# Build
In the project folder, run:  
```
cargo build --release
```
after this you'll get an executable in `./target/release/alby`

# Debug
```
cargo run
```

# Command-line options

* `--log-file`, `--log_file`, `--l`    
* `--tor-dir`, `--tor_dir`, `--t`

## Run with option

Executable:  

```
/some/folder/alby --log-file=/tmp/alby.log
```

Debug:  

```
cargo run -- --log-file=/tmp/alby.log
```
