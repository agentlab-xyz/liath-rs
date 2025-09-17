# System Dependencies

Liath depends on a C toolchain and a few common libraries.

Ubuntu/Debian:
```bash
sudo apt-get update
sudo apt-get install -y build-essential clang cmake libclang-dev libssl-dev pkg-config
```

CentOS/RHEL/Fedora:
```bash
sudo yum install -y gcc gcc-c++ make cmake clang openssl-devel
```

macOS:
```bash
# Xcode command line tools
xcode-select --install

# With Homebrew
brew install cmake openssl@3
```
