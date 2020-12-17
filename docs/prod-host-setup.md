# Production Host Setup Recommendations

## Firecracker Configuration

### Seccomp

Firecracker uses
[seccomp](https://www.kernel.org/doc/Documentation/prctl/seccomp_filter.txt)
filters to limit the system calls allowed by the host OS to the required
minimum.

By default, Firecracker uses advanced filtering, which is the most restrictive
option, and the recommended setting for production workloads.
This can also be explicitly requested by supplying `--seccomp-level=2` to the
Firecracker executable.

## Jailer Configuration

Using Jailer in a production Firecracker deployment is highly recommended,
as it provides additional security boundaries for the microVM.
The Jailer process applies
[cgroup](https://www.kernel.org/doc/Documentation/cgroup-v1/cgroups.txt),
namespace isolation and drops privileges of the Firecracker process.

To set up the jailer correctly, you'll need to:

- Create a dedicated non-privileged POSIX user and group to run Firecracker
  under. Use the created POSIX user and group IDs in Jailer's ``--uid <uid>``
  and ``--gid <gid>`` flags, respectively. This will run the Firecracker as
  the created non-privileged user and group. All file system resources used for
  Firecracker should be owned by this user and group. Apply least privilege to
  the resource files owned by this user and group to prevent other accounts from
  unauthorized file access.
  When running multiple Firecracker instances it is recommended that each runs
  with its unique `uid` and `gid` to provide an extra layer of security for
  their individually owned resources in the unlikely case where any one of the
  jails is broken out of.

Additional details of Jailer features can be found in the
[Jailer documentation](jailer.md).

## Host Security Configuration

### Mitigating Side-Channel Issues

When deploying Firecracker microVMs to handle multi-tenant workloads, the
following host environment configurations are strongly recommended to guard
against side-channel security issues.

Some of the mitigations are platform specific. When applicable, this information 
will be specified between brackets.

#### Disable Simultaneous Multithreading (SMT)

Disabling SMT will help mitigate side-channels issues between sibling
threads on the same physical core.

SMT can be disabled by adding the following Kernel boot parameter to the host:

```
nosmt=force
````

Verification can be done by running:

```bash
(grep -q "^forceoff$" /sys/devices/system/cpu/smt/control && \
echo "Hyperthreading: DISABLED (OK)") || \
(grep -q "^notsupported$\|^notimplemented$" /sys/devices/system/cpu/smt/control && \
echo "Hyperthreading: Not Supported (OK)") || \
echo "Hyperthreading: ENABLED (Recommendation: DISABLED)"
```

**Note** There are some newer aarch64 CPUs that also implement SMT, however AWS Graviton
processors do not implement it.

#### [Intel and ARM only] Check Kernel Page-Table Isolation (KPTI) support

KPTI is used to prevent certain side-channel issues that allow access to
protected kernel memory pages that are normally inaccessible to guests. Some
variants of Meltdown can be mitigated by enabling this feature.

Verification can be done by running:

```bash
(grep -q "^Mitigation: PTI$" /sys/devices/system/cpu/vulnerabilities/meltdown \
&& echo "KPTI: SUPPORTED (OK)") || \
(grep -q "^Not affected$" /sys/devices/system/cpu/vulnerabilities/meltdown \
&& echo "KPTI: Not Affected (OK)") || \
echo "KPTI: NOT SUPPORTED (Recommendation: SUPPORTED)"
```

A full list of the ARM processors that are vulnerable to side-channel attacks and
the mechanisms of these attacks can be found
[here](https://developer.arm.com/support/arm-security-updates/speculative-processor-vulnerability).
KPTI is implemented for ARM in version 4.16 and later of the Linux kernel.

**Note** Graviton-enabled hardware is not affected by this.

#### Disable Kernel Same-page Merging (KSM)

Disabling KSM mitigates side-channel issues which rely on de-duplication to
reveal what memory line was accessed by another process.

KSM can be disabled by executing the following as root:

```
echo "0" > /sys/kernel/mm/ksm/run
```

Verification can be done by running:

```bash
(grep -q "^0$" /sys/kernel/mm/ksm/run && echo "KSM: DISABLED (OK)") || \
echo "KSM: ENABLED (Recommendation: DISABLED)"
```

#### Check for mitigations against Spectre Side Channels

##### Branch Target Injection mitigation (Spectre V2)

**Intel and AMD** Use a kernel compiled with retpoline and run on hardware with microcode
supporting conditional Indirect Branch Prediction Barriers (IBPB) and
Indirect Branch Restricted Speculation (IBRS).

Verification can be done by running:

```bash
(grep -Eq '^Mitigation: Full [[:alpha:]]+ retpoline, IBPB: conditional, IBRS_FW' \
/sys/devices/system/cpu/vulnerabilities/spectre_v2 && \
echo "retpoline, IBPB, IBRS: ENABLED (OK)") \
|| echo "retpoline, IBPB, IBRS: DISABLED (Recommendation: ENABLED)"
```

**ARM** The mitigations for ARM systems are patched in all linux stable versions
starting with 4.16. More information on the processors vulnerable to this type
of attack and detailed information on the mitigations can be found in the
[ARM security documentation](https://developer.arm.com/support/arm-security-updates/speculative-processor-vulnerability).

Verification can be done by running:

```bash
(grep -q "^Mitigation:" /sys/devices/system/cpu/vulnerabilities/spectre_v2 || \
grep -q "^Not affected$" /sys/devices/system/cpu/vulnerabilities/spectre_v2) && \
echo "SPECTRE V2 -> OK" || echo "SPECTRE V2 -> NOT OK"
```

##### Bounds Check Bypass Store (Spectre V1)

Verification for mitigation against Spectre V1 can be done:

```bash
(grep -q "^Mitigation:" /sys/devices/system/cpu/vulnerabilities/spectre_v1 || \
grep -q "^Not affected$" /sys/devices/system/cpu/vulnerabilities/spectre_v1) && \
echo "SPECTRE V1 -> OK" || echo "SPECTRE V1 -> NOT OK"
```

#### [Intel only] Apply L1 Terminal Fault (L1TF) mitigation

These features provide mitigation for Foreshadow/L1TF side-channel issue on
affected hardware.

They can be enabled by adding the following Linux kernel boot parameter:

```
l1tf=full,force
```

which will also implicitly disable SMT.  This will apply the mitigation when
execution context switches into microVMs.

Verification can be done by running:

```bash
declare -a CONDITIONS=("Mitigation: PTE Inversion" "VMX: cache flushes")
for cond in "${CONDITIONS[@]}"; \
do (grep -q "$cond" /sys/devices/system/cpu/vulnerabilities/l1tf && \
echo "$cond: ENABLED (OK)") || \
echo "$cond: DISABLED (Recommendation: ENABLED)"; done
```

See more details [here](https://www.kernel.org/doc/html/latest/admin-guide/hw-vuln/l1tf.html#guest-mitigation-mechanisms).

#### Apply Speculative Store Bypass (SSBD) mitigation

This will mitigate variants of Spectre side-channel issues such as
Speculative Store Bypass and SpectreNG.

It can be enabled by adding the following Linux kernel boot parameter:

```
spec_store_bypass_disable=seccomp
```

which will apply SSB if seccomp is enabled by Firecracker.

Verification can be done by running:

```bash
cat /proc/$(pgrep firecracker | head -n1)/status | grep Speculation_Store_Bypass
```

Output shows one of the following:

- vulnerable
- not vulnerable
- thread mitigated
- thread force mitigated
- globally mitigated

#### Use memory with Rowhammer mitigation support

Rowhammer is a memory side-channel issue that can lead to unauthorized cross-
process memory changes.

Using DDR4 memory that supports Target Row Refresh (TRR) with error-correcting
code (ECC) is recommended. Use of pseudo target row refresh (pTRR) for systems
with pTRR-compliant DDR3 memory can help mitigate the issue, but it also
incurs a performance penalty.

#### Disable swapping to disk or enable secure swap

Memory pressure on a host can cause memory to be written to drive storage when
swapping is enabled. Disabling swap mitigates data remanence issues related to
having guest memory contents on microVM storage devices.

Verify that swap is disabled by running:

```bash
grep -q "/dev" /proc/swaps && echo "swap partitions present (Recommendation: no swap)" || echo "no swap partitions (OK)"
```

### Known kernel issues

General recommendation: Keep the host and the guest kernels up to date.

#### [CVE-2019-3016](https://nvd.nist.gov/vuln/detail/CVE-2019-3016)

##### Description

In a Linux KVM guest that has PV TLB enabled, a process in the guest kernel 
may be able to read memory locations from another process in the same guest.

##### Impact

Under certain conditions the TLB will contain invalid entries. A malicious 
attacker running on the guest can get access to the memory of other running 
process on that guest.

##### Vulnerable systems

The vulnerability affects systems where all the following conditions
are present:

- the host kernel >= 4.10.
- the guest kernel >= 4.16.
- the `KVM_FEATURE_PV_TLB_FLUSH` is set in the CPUID of the
guest. This is the `EAX` bit 9 in the `KVM_CPUID_FEATURES (0x40000001)` entry. 

This can be checked by running 

```bash
cpuid -r
```

and by searching for the entry corresponding to the leaf `0x40000001`.

Example output:

```
0x40000001 0x00: eax=0x200 ebx=0x00000000 ecx=0x00000000 edx=0x00000000
EAX 010004fb = 0010 0000 0000
EAX Bit 9: KVM_FEATURE_PV_TLB_FLUSH = 1 
```

##### Mitigation

The vulnerability is fixed by the following host kernel 
[patches](https://lkml.org/lkml/2020/1/30/482).

The fix was integrated in the mainline kernel and in 4.19.103, 5.4.19, 5.5.3
stable kernel releases. Please follow [kernel.org](https://www.kernel.org/) and
once the fix is available in your stable release please update the host kernel. 
If you are not using a vanilla kernel, please check with Linux distro provider.
