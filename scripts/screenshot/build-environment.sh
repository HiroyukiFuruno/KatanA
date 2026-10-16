#!/usr/bin/env bash

# 実測したmacOSだけJustと揃え、他OSのCargo target flagsを上書きしない。
case "${OSTYPE}" in
    darwin*) export RUSTFLAGS="${RUSTFLAGS--D warnings}" ;;
esac
