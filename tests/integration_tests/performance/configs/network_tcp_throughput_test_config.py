# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Configuration file for the network TCP throughput test."""

# pylint: disable=C0302
from framework.statistics.types import DefaultMeasurement

DEBUG = False

IPERF3 = "iperf3"
THROUGHPUT = "throughput"
THROUGHPUT_TOTAL = "total"
DURATION = "duration"
DURATION_TOTAL = "total"
RETRANSMITS = "retransmits"
RETRANSMITS_TOTAL = "total"
BASE_PORT = 5000
CPU_UTILIZATION_VMM = \
    f"{DefaultMeasurement.CPU_UTILIZATION_VMM.name.lower()}"
CPU_UTILIZATION_VCPUS_TOTAL = \
    f"{DefaultMeasurement.CPU_UTILIZATION_VCPUS_TOTAL.name.lower()}"
IPERF3_CPU_UTILIZATION_PERCENT_OUT_TAG = "cpu_utilization_percent"
IPERF3_END_RESULTS_TAG = "end"
DEBUG_CPU_UTILIZATION_VMM_SAMPLES_TAG = "cpu_utilization_vmm_samples"
DELTA_PERCENTAGE_TAG = "delta_percentage"
TARGET_TAG = "target"

CONFIG = {
    "time": 20,  # seconds
    "load_factor": 1,
    "modes": {
        "g2h": [""],
        "h2g": ["-R"],
        "bd": ["", "-R"]
    },
    "protocols": [
        {
            "name": "tcp",
            "omit": 5,
            "window_size": ["16K", "256K", None],
            "payload_length": ["1024K", None],
        }
    ],
    "hosts": {
        "instances": {
            "m5d.metal": {
                "cpus": [
                    {
                        "model": "Intel(R) Xeon(R) Platinum 8259CL CPU @ "
                                 "2.50GHz",
                        "throughput": {
                            "vmlinux-4.14.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 3359,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 25813,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 28051,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 3363,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 25835,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 28806,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 4144,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 23533,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 30387,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 4147,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 25834,
                                        "delta_percentage": 15
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 33251,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 4164,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 26737,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 30161,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 4164,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 27379,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 31033,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 2976,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 20375,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 27174,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 2973,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 20379,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 28513,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 2619,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 14510,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 31480,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 2620,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 16999,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 33932,
                                        "delta_percentage": 5
                                    }
                                }
                            },
                            "vmlinux-4.9.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 3114,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 25659,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 27725,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 3114,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 25661,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 28344,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 4202,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 23208,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 26071,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 4201,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 25986,
                                        "delta_percentage": 14
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 32578,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 3955,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 26075,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 29308,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 3945,
                                        "delta_percentage": 13
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 26777,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 30801,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 2857,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 20169,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 27132,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 2858,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 20180,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 27699,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 2563,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 14245,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 24591,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 2564,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 16910,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 33405,
                                        "delta_percentage": 5
                                    }
                                }
                            }
                        },
                        "cpu_utilization_vmm": {
                            "vmlinux-4.14.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 57,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 94,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 57,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 89,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 94,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 83,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 83,
                                        "delta_percentage": 13
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 58,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 91,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 94,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 58,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 91,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 93,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 49,
                                        "delta_percentage": 10
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 75,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 90,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 49,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 75,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 92,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 39,
                                        "delta_percentage": 11
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 56,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 87,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 39,
                                        "delta_percentage": 10
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 54,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 88,
                                        "delta_percentage": 6
                                    }
                                }
                            },
                            "vmlinux-4.9.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 88,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 93,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 93,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 51,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 82,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 84,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 84,
                                        "delta_percentage": 11
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 88,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 55,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 90,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 94,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 55,
                                        "delta_percentage": 14
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 92,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 47,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 74,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 90,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 47,
                                        "delta_percentage": 10
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 74,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 90,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 38,
                                        "delta_percentage": 11
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 55,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 78,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 38,
                                        "delta_percentage": 11
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 54,
                                        "delta_percentage": 8
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 87,
                                        "delta_percentage": 7
                                    }
                                }
                            }
                        },
                        "cpu_utilization_vcpus_total": {
                            "vmlinux-4.14.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 114,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 114,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 190,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 186,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 197,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 176,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 164,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    }
                                }
                            },
                            "vmlinux-4.9.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 114,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 118,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 197,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 181,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 184,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 175,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 164,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    }
                                }
                            }
                        }
                    },
                    {
                        "model": "Intel(R) Xeon(R) Platinum 8175M CPU @ "
                                 "2.50GHz",
                        "throughput": {
                            "vmlinux-4.14.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 2749,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 24153,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 26827,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 2756,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 24139,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 27491,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 3434,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 20573,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 27994,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 3439,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 24089,
                                        "delta_percentage": 15
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 31885,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 3236,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 23782,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 29144,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 3246,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 24308,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 29493,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 2521,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 18606,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 25446,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 2519,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 18638,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 26616,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 2207,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 13812,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 29579,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 2209,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 14839,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 31913,
                                        "delta_percentage": 7
                                    }
                                }
                            },
                            "vmlinux-4.9.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 2946,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 26944,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 27140,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 2948,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 26949,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 27722,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 4040,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 21646,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 26125,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 4046,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 23994,
                                        "delta_percentage": 11
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 33624,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 3723,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 25134,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 28180,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 3720,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 25933,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 29736,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 2582,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 19537,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 26151,
                                        "delta_percentage": 27
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 2580,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 19529,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 27303,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 2439,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 14706,
                                        "delta_percentage": 4
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 24763,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 2439,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 15958,
                                        "delta_percentage": 4
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 34569,
                                        "delta_percentage": 6
                                    }
                                }
                            }
                        },
                        "cpu_utilization_vmm": {
                            "vmlinux-4.14.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 56,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 94,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 56,
                                        "delta_percentage": 8
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 94,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 80,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 87,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 84,
                                        "delta_percentage": 13
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 57,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 95,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 57,
                                        "delta_percentage": 8
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 89,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 93,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 51,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 74,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 90,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 51,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 74,
                                        "delta_percentage": 8
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 91,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 40,
                                        "delta_percentage": 10
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 59,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 86,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 40,
                                        "delta_percentage": 10
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 86,
                                        "delta_percentage": 7
                                    }
                                }
                            },
                            "vmlinux-4.9.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 57,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 97,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 95,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 57,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 97,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 95,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 60,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 84,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 88,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 61,
                                        "delta_percentage": 10
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 84,
                                        "delta_percentage": 10
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 92,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 62,
                                        "delta_percentage": 9
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 92,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 95,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 62,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 92,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 93,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 51,
                                        "delta_percentage": 10
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 78,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 91,
                                        "delta_percentage": 26
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 51,
                                        "delta_percentage": 9
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 78,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 93,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 44,
                                        "delta_percentage": 10
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 60,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 82,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 44,
                                        "delta_percentage": 10
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 56,
                                        "delta_percentage": 8
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 90,
                                        "delta_percentage": 7
                                    }
                                }
                            }
                        },
                        "cpu_utilization_vcpus_total": {
                            "vmlinux-4.14.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 197,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 121,
                                        "delta_percentage": 8
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 119,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 185,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 184,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 181,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 197,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 165,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    }
                                }
                            },
                            "vmlinux-4.9.bin": {
                                "ubuntu-18.04.ext4": {
                                    "tcp-pDEFAULT-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-g2h": {
                                        "target": 117,
                                        "delta_percentage": 8
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-g2h": {
                                        "target": 112,
                                        "delta_percentage": 8
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-h2g": {
                                        "target": 197,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-h2g": {
                                        "target": 182,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-h2g": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-h2g": {
                                        "target": 187,
                                        "delta_percentage": 7
                                    },
                                    "tcp-pDEFAULT-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-2vcpu-bd": {
                                        "target": 174,
                                        "delta_percentage": 6
                                    },
                                    "tcp-p1024K-ws16K-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-2vcpu-bd": {
                                        "target": 197,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-2vcpu-bd": {
                                        "target": 163,
                                        "delta_percentage": 6
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 7
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-pDEFAULT-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws16K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-ws256K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "tcp-p1024K-wsDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    }
                                }
                            }
                        }
                    }
                ]
            }
        }
    }
}
