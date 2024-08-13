#!/bin/bash
# -*- shell-script -*-
# Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

# ./tools/devtool shell --privileged
# cargo install bindgen-cli
# apt update && apt install patch
# ./tools/bindgen.sh

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
    non_snake_case,
    clippy::ptr_as_ptr,
    clippy::undocumented_unsafe_blocks,
    missing_debug_implementations,
    clippy::tests_outside_test_module
)]

EOF
    bindgen --no-doc-comments --disable-header-comment --constified-enum '*' --with-derive-default --with-derive-partialeq $@
}

KERNEL_HEADERS_HOME="/usr"

info "BINDGEN sockios.h"
fc-bindgen "$KERNEL_HEADERS_HOME/include/linux/sockios.h" |replace_linux_int_types >src/vmm/src/devices/virtio/net/gen/sockios.rs

info "BINDGEN if.h"
fc-bindgen "$KERNEL_HEADERS_HOME/include/linux/if.h" \
           --allowlist-var='IF.*' \
           --allowlist-type='if.*' \
           --allowlist-type="net_device.*" \
           -- -D __UAPI_DEF_IF_IFNAMSIZ -D __UAPI_DEF_IF_NET_DEVICE_FLAGS -D __UAPI_DEF_IF_IFREQ -D __UAPI_DEF_IF_IFMAP >src/vmm/src/devices/virtio/net/gen/iff.rs

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
   "$KERNEL_HEADERS_HOME/include/linux/if_tun.h" >src/vmm/src/devices/virtio/net/gen/if_tun.rs

info "BINDGEN virtio_ring.h"
fc-bindgen \
    --allowlist-var "VIRTIO_RING_F_EVENT_IDX" \
    "$KERNEL_HEADERS_HOME/include/linux/virtio_ring.h" >src/vmm/src/devices/virtio/gen/virtio_ring.rs

info "BINDGEN virtio_blk.h"
fc-bindgen \
    --allowlist-var "VIRTIO_BLK_.*" \
    --allowlist-var "VIRTIO_F_.*" \
    "$KERNEL_HEADERS_HOME/include/linux/virtio_blk.h" >src/vmm/src/devices/virtio/gen/virtio_blk.rs

info "BINDGEN virtio_net.h"
fc-bindgen \
    --allowlist-var "VIRTIO_NET_F_.*" \
    --allowlist-var "VIRTIO_F_.*" \
    --allowlist-type "virtio_net_hdr_v1" \
    "$KERNEL_HEADERS_HOME/include/linux/virtio_net.h" >src/vmm/src/devices/virtio/gen/virtio_net.rs

info "BINDGEN virtio_rng.h"
fc-bindgen \
    --allowlist-var "VIRTIO_RNG_.*" \
    --allowlist-var "VIRTIO_F_.*" \
    "$KERNEL_HEADERS_HOME/include/linux/virtio_rng.h" >src/vmm/src/devices/virtio/gen/virtio_rng.rs

info "BINDGEN prctl.h"
fc-bindgen \
    --allowlist-var "PR_.*" \
    "$KERNEL_HEADERS_HOME/include/linux/prctl.h" >src/firecracker/src/gen/prctl.rs
sed -i '/PR_SET_SPECULATION_CTRL/s/u32/i32/g' src/firecracker/src/gen/prctl.rs

# https://www.kernel.org/doc/Documentation/kbuild/headers_install.txt
# The Linux repo is huge. Just copy what we need.
# git clone --branch v5.10 --depth 1 https://github.com/torvalds/linux.git linux
git clone --branch linux-5.10.y --depth 1 https://github.com/amazonlinux/linux amazonlinux-v5.10.y

info "BINDGEN mpspec_def.h"
fc-bindgen amazonlinux-v5.10.y/arch/x86/include/asm/mpspec_def.h \
           >src/vmm/src/arch/x86_64/gen/mpspec.rs
# https://github.com/rust-lang/rust-bindgen/issues/1274

info "BINDGEN msr-index.h"
cp -r amazonlinux-v5.10.y/include/asm-generic amazonlinux-v5.10.y/include/asm
sed -i -E 's/__no_(sanitize|kasan)_or_inline//g' amazonlinux-v5.10.y/include/asm/rwonce.h
fc-bindgen amazonlinux-v5.10.y/arch/x86/include/asm/msr-index.h \
    --allowlist-var "^MSR_.*$" \
    -- \
    -Iamazonlinux-v5.10.y/include/ \
    -Iamazonlinux-v5.10.y/arch/x86/include/ \
    -Wno-macro-redefined \
    >src/vmm/src/arch/x86_64/gen/msr_index.rs
perl -i -pe 's/= (\d+);/sprintf("= 0x%x;",$1)/eg' src/vmm/src/arch/x86_64/gen/msr_index.rs

info "BINDGEN perf_event.h"
grep "MSR_ARCH_PERFMON_" amazonlinux-v5.10.y/arch/x86/include/asm/perf_event.h \
    >amazonlinux-v5.10.y/arch/x86/include/asm/perf_event_msr.h
fc-bindgen amazonlinux-v5.10.y/arch/x86/include/asm/perf_event_msr.h \
    --allowlist-var "^MSR_ARCH_PERFMON_.*$" \
    -- \
    >src/vmm/src/arch/x86_64/gen/perf_event.rs
perl -i -pe 's/= (\d+);/sprintf("= 0x%x;",$1)/eg' src/vmm/src/arch/x86_64/gen/perf_event.rs

info "BINDGEN hyperv.h"
grep "HV_X64_MSR_" amazonlinux-v5.10.y/arch/x86/kvm/hyperv.h \
    >amazonlinux-v5.10.y/arch/x86/kvm/hyperv_msr.h
fc-bindgen amazonlinux-v5.10.y/arch/x86/kvm/hyperv_msr.h \
    --allowlist-var "^HV_X64_MSR_.*$" \
    -- \
    >src/vmm/src/arch/x86_64/gen/hyperv.rs
perl -i -pe 's/= (\d+);/sprintf("= 0x%x;",$1)/eg' src/vmm/src/arch/x86_64/gen/hyperv.rs

info "BINDGEN hyperv-tlfs.h"
grep "HV_X64_MSR_" amazonlinux-v5.10.y/arch/x86/include/asm/hyperv-tlfs.h \
    >amazonlinux-v5.10.y/arch/x86/include/asm/hyperv-tlfs_msr.h
fc-bindgen amazonlinux-v5.10.y/arch/x86/include/asm/hyperv-tlfs_msr.h \
    --allowlist-var "^HV_X64_MSR_.*$" \
    -- \
    >src/vmm/src/arch/x86_64/gen/hyperv_tlfs.rs
perl -i -pe 's/= (\d+);/sprintf("= 0x%x;",$1)/eg' src/vmm/src/arch/x86_64/gen/hyperv_tlfs.rs

info "BINDGEN io_uring.h"
fc-bindgen \
    --allowlist-var "IORING_.+" \
    --allowlist-var "IO_URING_.+" \
    --allowlist-var "IOSQE_.+" \
    --allowlist-type "io_uring_.+" \
    --allowlist-type "io_.qring_offsets" \
    "amazonlinux-v5.10.y/include/uapi/linux/io_uring.h" \
    >src/vmm/src/io_uring/gen.rs

# Apply any patches
info "Apply patches"
for PATCH in $(dirname $0)/bindgen-patches/*.patch; do
    git apply $PATCH
done

echo "Bindings created correctly! You might want to run ./tools/test_bindings.py to test for ABI incompatibilities"
