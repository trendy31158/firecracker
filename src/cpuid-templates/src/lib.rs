// Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use cpuid::{Cpuid, CpuidEntry, CpuidKey, CpuidRegisters, IntelCpuid, KvmCpuidFlags};

pub fn c3() -> Cpuid {
    Cpuid::Intel(IntelCpuid({
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            CpuidKey {
                leaf: 0u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 13u32,
                    ebx: 1970169159u32,
                    ecx: 1818588270u32,
                    edx: 1231384169u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 198372u32,
                    ebx: 133120u32,
                    ecx: 4156170755u32,
                    edx: 395049983u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1979933441u32,
                    ebx: 15775231u32,
                    ecx: 0u32,
                    edx: 12779520u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 3u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109153u32,
                    ebx: 29360191u32,
                    ecx: 63u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109154u32,
                    ebx: 29360191u32,
                    ecx: 63u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109187u32,
                    ebx: 62914623u32,
                    ecx: 1023u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 3u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67125603u32,
                    ebx: 41943103u32,
                    ecx: 53247u32,
                    edx: 5u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 4u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67108864u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 5u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 6u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 4u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 7u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 1049219u32,
                    ecx: 0u32,
                    edx: 2885682176u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 8u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 9u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 10u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 1u32,
                    ecx: 256u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 7u32,
                    ebx: 2u32,
                    ecx: 513u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 2u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 12u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 7u32,
                    ebx: 2696u32,
                    ecx: 2696u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 1u32,
                    ebx: 2568u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 256u32,
                    ebx: 576u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 3u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 960u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 4u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 1024u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 5u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 1088u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 6u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 512u32,
                    ebx: 1152u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 7u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 1024u32,
                    ebx: 1664u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 9u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 8u32,
                    ebx: 2688u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1073741824u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1073741825u32,
                    ebx: 1263359563u32,
                    ecx: 1447775574u32,
                    edx: 77u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1073741825u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 16809723u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483648u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 2147483656u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483649u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 1u32,
                    edx: 672139264u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483650u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1702129225u32,
                    ebx: 693250156u32,
                    ecx: 1868912672u32,
                    edx: 693250158u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483651u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1869762592u32,
                    ebx: 1936942435u32,
                    ecx: 1075868271u32,
                    edx: 808334112u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483652u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 2051557168u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483653u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483654u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 16801856u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483655u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 256u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483656u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 12334u32,
                    ebx: 16830464u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map
    }))
}

pub fn t2() -> Cpuid {
    Cpuid::Intel(IntelCpuid({
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            CpuidKey {
                leaf: 0u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 13u32,
                    ebx: 1970169159u32,
                    ecx: 1818588270u32,
                    edx: 1231384169u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 198386u32,
                    ebx: 133120u32,
                    ecx: 4160369155u32,
                    edx: 395049983u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1979933441u32,
                    ebx: 15775231u32,
                    ecx: 0u32,
                    edx: 12779520u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 3u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109153u32,
                    ebx: 29360191u32,
                    ecx: 63u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109154u32,
                    ebx: 29360191u32,
                    ecx: 63u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109187u32,
                    ebx: 62914623u32,
                    ecx: 1023u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 3u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67125603u32,
                    ebx: 41943103u32,
                    ecx: 53247u32,
                    edx: 5u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 4u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67108864u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 5u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 6u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 4u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 7u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 1050539u32,
                    ecx: 0u32,
                    edx: 2885682176u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 8u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 9u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 10u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 1u32,
                    ecx: 256u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 7u32,
                    ebx: 2u32,
                    ecx: 513u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 2u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 12u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 7u32,
                    ebx: 2696u32,
                    ecx: 2696u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 1u32,
                    ebx: 2568u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 256u32,
                    ebx: 576u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 3u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 960u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 4u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 1024u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 5u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 1088u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 6u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 512u32,
                    ebx: 1152u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 7u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 1024u32,
                    ebx: 1664u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 9u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 8u32,
                    ebx: 2688u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1073741824u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1073741825u32,
                    ebx: 1263359563u32,
                    ecx: 1447775574u32,
                    edx: 77u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1073741825u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 16809723u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483648u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 2147483656u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483649u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 33u32,
                    edx: 672139264u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483650u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1702129225u32,
                    ebx: 693250156u32,
                    ecx: 1868912672u32,
                    edx: 693250158u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483651u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1869762592u32,
                    ebx: 1936942435u32,
                    ecx: 1075868271u32,
                    edx: 808334112u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483652u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 2051557168u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483653u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483654u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 16801856u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483655u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 256u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483656u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 12334u32,
                    ebx: 16830464u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map
    }))
}

pub fn t2s() -> Cpuid {
    Cpuid::Intel(IntelCpuid({
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            CpuidKey {
                leaf: 0u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 13u32,
                    ebx: 1970169159u32,
                    ecx: 1818588270u32,
                    edx: 1231384169u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 198386u32,
                    ebx: 133120u32,
                    ecx: 4160369155u32,
                    edx: 395049983u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1979933441u32,
                    ebx: 15775231u32,
                    ecx: 0u32,
                    edx: 12779520u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 3u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109153u32,
                    ebx: 29360191u32,
                    ecx: 63u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109154u32,
                    ebx: 29360191u32,
                    ecx: 63u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67109187u32,
                    ebx: 62914623u32,
                    ecx: 1023u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 3u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67125603u32,
                    ebx: 41943103u32,
                    ecx: 53247u32,
                    edx: 5u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 4u32,
                subleaf: 4u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 67108864u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 5u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 6u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 4u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 7u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 1050539u32,
                    ecx: 0u32,
                    edx: 2885682176u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 8u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 9u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 10u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 1u32,
                    ecx: 256u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 7u32,
                    ebx: 2u32,
                    ecx: 513u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 11u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 2u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 12u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 7u32,
                    ebx: 2696u32,
                    ecx: 2696u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 1u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 1u32,
                    ebx: 2568u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 2u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 256u32,
                    ebx: 576u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 3u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 960u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 4u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 1024u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 5u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 64u32,
                    ebx: 1088u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 6u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 512u32,
                    ebx: 1152u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 7u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 1024u32,
                    ebx: 1664u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 13u32,
                subleaf: 9u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(1u32),
                result: CpuidRegisters {
                    eax: 8u32,
                    ebx: 2688u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1073741824u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1073741825u32,
                    ebx: 1263359563u32,
                    ecx: 1447775574u32,
                    edx: 77u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 1073741825u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 16809723u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483648u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 2147483656u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483649u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 33u32,
                    edx: 672139264u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483650u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1702129225u32,
                    ebx: 693250156u32,
                    ecx: 1868912672u32,
                    edx: 693250158u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483651u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 1869762592u32,
                    ebx: 1936942435u32,
                    ecx: 1075868271u32,
                    edx: 808334112u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483652u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 2051557168u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483653u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483654u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 16801856u32,
                    edx: 0u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483655u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 0u32,
                    ebx: 0u32,
                    ecx: 0u32,
                    edx: 256u32,
                },
            },
        );
        map.insert(
            CpuidKey {
                leaf: 2147483656u32,
                subleaf: 0u32,
            },
            CpuidEntry {
                flags: KvmCpuidFlags(0u32),
                result: CpuidRegisters {
                    eax: 12334u32,
                    ebx: 16830464u32,
                    ecx: 0u32,
                    edx: 0u32,
                },
            },
        );
        map
    }))
}
