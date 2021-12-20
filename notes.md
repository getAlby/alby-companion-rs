# Alby native companion app

Alby is a Browser extension for the Lightning Network. It can connect to user's lightning nodes and facilitates lightning payments online.


Alby does this by directly connecting to the user's node through normal HTTP requests.
Many users run nodes behind Tor or using self-signed certificates.
Browsers can not natively handle those requests easily.

To make those connections possible we plan to implement a native companion application.
That native application runs natively on the user's computer. The browser extension communicates through [native messaging](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging) with the application.

The application performs the request and returns the response to the browser extension.


## ToDos:


### Initiate Project and integrate libtor
* Initiate a new rust project
* Embed Tor (libtor) and start a Tor connection
	* Does this require to run a socks proxy?
	* Run on a random port and plan for authentication (the socks proxy should not be available to other apps)
* Perform a request through Tor

### Connect to web extension
* Read JSON data from STDIN (https://github.com/neon64/chrome-native-messaging)
* Write JSON data to STDOUT
* Initiate request from the web extension

### Add support for non-Tor HTTP requests
* Allow HTTP requests to self-sign certificates
*


## Links

* https://github.com/MagicalBitcoin/libtor
* https://github.com/MagicalBitcoin/libtor-sys
* https://github.com/neon64/chrome-native-messaging
* https://github.com/Jikstra/torshare (using libtor to download/share files)
* https://github.com/tauri-apps/tauri (for a potential UI?)
* https://github.com/mrmekon/connectr (inspiation for a tab icon application)
* https://github.com/delta1/libtor-rs-example (libtor example)
