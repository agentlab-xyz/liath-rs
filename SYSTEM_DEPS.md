# System Dependencies

Liath requires certain system dependencies to be installed:

## Ubuntu/Debian
```bash
sudo apt-get update
sudo apt-get install -y build-essential clang cmake libclang-dev libssl-dev pkg-config
```

## CentOS/RHEL/Fedora
```bash
sudo yum install -y gcc gcc-c++ make cmake clang openssl-devel
```

## macOS
```bash
# Install Xcode command line tools
xcode-select --install

# Install with Homebrew
brew install cmake openssl@3
```