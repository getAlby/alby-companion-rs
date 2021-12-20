# Setup
## MacOS
You'll need [Homebrew](https://brew.sh/)  
  
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
