# LLVM 18.1.0 Compilation


## Download LLVM 18.1.0

```shell
wget https://github.com/llvm/llvm-project/archive/refs/tags/llvmorg-18.1.0-rc1.tar.gz
tar xvzf llvmorg-18.1.0-rc1.tar.gz  # `x` extract, `v` verbose, `z` gunzip, `f` file
```

## Set up the build directory

```shell
cd llvm-project-llvmorg-18.1.0-rc1
cmake -S llvm -B build -G Ninja -DCMAKE_INSTALL_PREFIX=$HOME/llvm-18.1.0 -DCMAKE_BUILD_TYPE=debug
```

Optionally: `-DLLVM_ENABLE_PROJECTS="clang;lld"`

## Build LLVM

```shell
ninja -C build check-llvm
```

## Install LLVM

```shell
ninja -C build install
```

# DON'T USE `cmake` to build and install LLVM

`cmake` 3.18 has a [bug][cmake-issue] ([LLVM issue][llvm-issue]) that causes the build to fail.

We found that `cmake` 3.31 still has the same issue.


The following commands will fail with these versions of `cmake`:

```shell
cmake --build build --target all -j$(nproc)
cmake --install build
```


[cmake-issue]: https://gitlab.kitware.com/cmake/cmake/-/issues/24647
[llvm-issue]: https://github.com/llvm/llvm-project/issues/61738
