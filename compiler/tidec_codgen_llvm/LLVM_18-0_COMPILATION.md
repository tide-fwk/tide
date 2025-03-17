# LLVM 18.1.0 Compilation


## Download LLVM 18.1.0

```shell
wget https://github.com/llvm/llvm-project/archive/refs/tags/llvmorg-18.1.0-rc1.tar.gz
tar xvzf llvmorg-18.1.0-rc1.tar.gz  # `x` extract, `v` verbose, `z` gunzip, `f` file
```

## Set up the build directory

```shell
cd llvm-project-llvmorg-18.1.0-rc1
cmake -S llvm -B build -DCMAKE_INSTALL_PREFIX=$HOME/llvm-18.1.0 -DCMAKE_BUILD_TYPE=debug
```

Optionally: `-DLLVM_ENABLE_PROJECTS="clang;lld"`

## Build LLVM

```shell
cmake --build build --target all -j$(nproc)
```

## Install LLVM

```shell
cmake --install build
```
