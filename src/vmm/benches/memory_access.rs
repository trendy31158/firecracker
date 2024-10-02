// Copyright 2024 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use vm_memory::GuestMemory;
use vmm::resources::VmResources;
use vmm::vmm_config::machine_config::{HugePageConfig, VmConfig};

fn bench_single_page_fault(c: &mut Criterion, configuration: VmResources) {
    c.bench_function("page_fault", |b| {
        b.iter_batched(
            || {
                let memory = configuration.allocate_guest_memory().unwrap();
                // Get a pointer to the first memory region (cannot do `.get_slice(GuestAddress(0),
                // 1)`, because on ARM64 guest memory does not start at physical
                // address 0).
                let ptr = memory.iter().next().unwrap().as_ptr();

                // fine to return both here, because ptr is not a reference into `memory` (e.g. no
                // self-referential structs are happening here)
                (memory, ptr)
            },
            |(_, ptr)| unsafe {
                // Cause a single page fault
                ptr.write_volatile(1);
            },
            BatchSize::SmallInput,
        )
    });
}

pub fn bench_4k_page_fault(c: &mut Criterion) {
    bench_single_page_fault(
        c,
        VmResources {
            vm_config: VmConfig {
                vcpu_count: 1,
                mem_size_mib: 2,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

pub fn bench_2m_page_fault(c: &mut Criterion) {
    bench_single_page_fault(
        c,
        VmResources {
            vm_config: VmConfig {
                vcpu_count: 1,
                mem_size_mib: 2,
                huge_pages: HugePageConfig::Hugetlbfs2M,
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

criterion_group! {
    name = memory_access_benches;
    config = Criterion::default().noise_threshold(0.05);
    targets = bench_4k_page_fault, bench_2m_page_fault
}

criterion_main! {
    memory_access_benches
}
