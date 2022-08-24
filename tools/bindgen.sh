#!/bin/bash
# -*- shell-script -*-
# Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

# ./tools/devtool shell --privileged
# bindgen-0.60 has a dependency that needs Rust edition 2021
# cargo +stable install bindgen
# apt update && apt install patch

set -eu

# Borrowed from crosvm https://chromium.googlesource.com/chromiumos/platform/crosvm/+/refs/heads/main/tools/impl/bindgen-common.sh#33
replace_linux_int_types() {
    sed -E -e '/^pub type __(u|s)(8|16|32|64) =/d' -e 's/__u(8|16|32|64)/u\1/g' -e 's/__s(8|16|32|64)/i\1/g'
}

function info {
    echo $@ >&2
}

function fc-bindgen {
    cat <<EOF
// Copyright $(date +%Y) Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

// automatically generated by tools/bindgen.sh

#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    dead_code,
    non_snake_case
)]

EOF
    bindgen --disable-header-comment --size_t-is-usize --constified-enum '*' --with-derive-default --with-derive-partialeq $@
}

KERNEL_HEADERS_HOME="/usr"

info "BINDGEN in.h"
fc-bindgen "$KERNEL_HEADERS_HOME/include/linux/in.h" \
           --allowlist-var='IP.*' \
           --allowlist-var='IN.*' \
           --allowlist-var='MCAST' \
           --allowlist-type='in_.*' \
           --allowlist-type='ip_.*' \
    |replace_linux_int_types >src/net_gen/src/inn.rs

info "BINDGEN sockios.h"
fc-bindgen "$KERNEL_HEADERS_HOME/include/linux/sockios.h" |replace_linux_int_types >src/net_gen/src/sockios.rs

info "BINDGEN if.h"
fc-bindgen "$KERNEL_HEADERS_HOME/include/linux/if.h" \
           --allowlist-var='IF.*' \
           --allowlist-type='if.*' \
           --allowlist-type="net_device.*" \
           -- -D __UAPI_DEF_IF_IFNAMSIZ -D __UAPI_DEF_IF_NET_DEVICE_FLAGS -D __UAPI_DEF_IF_IFREQ -D __UAPI_DEF_IF_IFMAP >src/net_gen/src/iff.rs

info "BINDGEN if_tun.h"
fc-bindgen \
    --allowlist-type='sock_fprog' \
    --allowlist-var='TUN_.*' \
    --allowlist-var='IFF_NO_PI' \
    --allowlist-var='IFF_MULTI_QUEUE' \
    --allowlist-var='IFF_TAP' \
    --allowlist-var='IFF_VNET_HDR' \
    --allowlist-var='ETH_.*' \
    --allowlist-type='ifreq' \
   "$KERNEL_HEADERS_HOME/include/linux/if_tun.h" >src/net_gen/src/if_tun.rs

info "BINDGEN virtio_ring.h"
fc-bindgen \
    --allowlist-var "VIRTIO_RING_F_EVENT_IDX" \
    "$KERNEL_HEADERS_HOME/include/linux/virtio_ring.h" >src/virtio_gen/src/virtio_ring.rs

info "BINDGEN virtio_blk.h"
fc-bindgen \
    --allowlist-var "VIRTIO_BLK_.*" \
    --allowlist-var "VIRTIO_F_.*" \
    "$KERNEL_HEADERS_HOME/include/linux/virtio_blk.h" >src/virtio_gen/src/virtio_blk.rs

info "BINDGEN virtio_net.h"
fc-bindgen \
    --allowlist-var "VIRTIO_NET_F_.*" \
    --allowlist-var "VIRTIO_F_.*" \
    --allowlist-type "virtio_net_hdr_v1" \
    "$KERNEL_HEADERS_HOME/include/linux/virtio_net.h" >src/virtio_gen/src/virtio_net.rs

# https://www.kernel.org/doc/Documentation/kbuild/headers_install.txt
# The Linux repo is huge. Just copy what we need.
# git clone --branch v5.10 --depth 1 https://github.com/torvalds/linux.git linux
git clone --branch linux-5.10.y --depth 1 https://github.com/amazonlinux/linux amazonlinux-v5.10.y

info "BINDGEN mpspec_def.h"
fc-bindgen amazonlinux-v5.10.y/arch/x86/include/asm/mpspec_def.h \
           >src/arch_gen/src/x86/mpspec.rs
# https://github.com/rust-lang/rust-bindgen/issues/1274

info "BINDGEN io_uring.h"
fc-bindgen \
    --allowlist-var "IORING_.+" \
    --allowlist-var "IO_URING_.+" \
    --allowlist-var "IOSQE_.+" \
    --allowlist-type "io_uring_.+" \
    --allowlist-type "io_.qring_offsets" \
    "amazonlinux-v5.10.y/include/uapi/linux/io_uring.h" \
    >src/io_uring/src/bindings.rs

# Apply any patches
# src/virtio_gen
for crate in src/net_gen; do
    for patch in $(dirname $0)/bindgen-patches/$(basename $crate)/*.patch; do
        echo PATCH $crate/$patch
        (cd $crate; patch -p1) <$patch
    done
done
