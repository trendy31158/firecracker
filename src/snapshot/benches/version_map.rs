// Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use snapshot::Snapshot;
use versionize::{VersionMap, Versionize, VersionizeResult};
use versionize_derive::Versionize;

#[derive(Clone, Debug, Default, Versionize)]
struct Test {
    a: Vec<Dummy>,
    #[version(start = 1)]
    b: u64,
    #[version(start = 2)]
    c: u64,
    #[version(start = 3)]
    d: u32,
    #[version(start = 4)]
    e: Vec<u64>,
}

#[derive(Clone, Debug, Default, Versionize)]
struct Dummy {
    a: String,
    #[version(start = 2)]
    b: [u64; 32],
}

#[inline]
#[tracing::instrument(level = "debug", ret(skip), skip(snapshot_mem, vm))]
fn restore(mut snapshot_mem: &[u8], vm: VersionMap) {
    Snapshot::unchecked_load::<&[u8], Test>(&mut snapshot_mem, vm).unwrap();
}

#[inline]
#[tracing::instrument(level = "debug", ret(skip), skip(snapshot_mem, vm))]
fn save<W: std::io::Write>(mut snapshot_mem: &mut W, vm: VersionMap) {
    let state = Test {
        a: vec![
            Dummy {
                a: "a string".to_owned(),
                b: [0x1234u64; 32]
            };
            200
        ],
        b: 0,
        c: 1,
        d: 2,
        e: vec![0x4321; 100],
    };

    let mut snapshot = Snapshot::new(vm.clone(), vm.latest_version());
    snapshot
        .save_without_crc(&mut snapshot_mem, &state)
        .unwrap();
}

#[tracing::instrument(level = "debug", ret(skip), skip(c))]
pub fn criterion_benchmark(c: &mut Criterion) {
    let mut snapshot_mem = vec![0u8; 1024 * 1024 * 128];
    let mut vm = VersionMap::new();

    vm.new_version()
        .set_type_version(Test::type_id(), 2)
        .new_version()
        .set_type_version(Test::type_id(), 3)
        .new_version()
        .set_type_version(Test::type_id(), 4)
        .set_type_version(Dummy::type_id(), 2);

    let mut slice = &mut snapshot_mem.as_mut_slice();
    save(&mut slice, vm.clone());
    let snapshot_len = slice.as_ptr() as usize - snapshot_mem.as_slice().as_ptr() as usize;
    println!("Snapshot length: {} bytes", snapshot_len);

    c.bench_function("Serialize in vspace=4", |b| {
        b.iter(|| {
            save(
                black_box(&mut snapshot_mem.as_mut_slice()),
                black_box(vm.clone()),
            )
        })
    });

    c.bench_function("Deserialize in vspace=4", |b| {
        b.iter(|| restore(black_box(snapshot_mem.as_slice()), black_box(vm.clone())))
    });

    // Extend vspace to 100.
    for _ in 0..96 {
        vm.new_version();
    }

    save(&mut snapshot_mem.as_mut_slice(), vm.clone());

    c.bench_function("Serialize in vspace=100", |b| {
        b.iter(|| {
            save(
                black_box(&mut snapshot_mem.as_mut_slice()),
                black_box(vm.clone()),
            )
        })
    });
    c.bench_function("Deserialize in vspace=100", |b| {
        b.iter(|| restore(black_box(snapshot_mem.as_slice()), black_box(vm.clone())))
    });

    // Extend vspace to 1001.
    for _ in 0..900 {
        vm.new_version();
    }

    // Save the snapshot at version 1001.
    save(&mut snapshot_mem.as_mut_slice(), vm.clone());

    c.bench_function("Serialize in vspace=1000", |b| {
        b.iter(|| {
            save(
                black_box(&mut snapshot_mem.as_mut_slice()),
                black_box(vm.clone()),
            )
        })
    });
    c.bench_function("Deserialize in vspace=1000", |b| {
        b.iter(|| restore(black_box(snapshot_mem.as_slice()), black_box(vm.clone())))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(200).noise_threshold(0.05);
    targets = criterion_benchmark
}

criterion_main! {
    benches
}
