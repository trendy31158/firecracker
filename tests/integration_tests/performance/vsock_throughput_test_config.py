# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Configuration file for the VSOCK throughput test."""

IPERF3 = "iperf3-vsock"
THROUGHPUT = "throughput"
THROUGHPUT_TOTAL = "total"
DURATION = "duration"
DURATION_TOTAL = "total"
CPU_UTILIZATION_HOST = "cpu_utilization_host"
CPU_UTILIZATION_GUEST = "cpu_utilization_guest"
BASE_PORT = 5201
CPU_UTILIZATION_VMM_TAG = "vmm"
CPU_UTILIZATION_VCPUS_TOTAL_TAG = "vcpus_total"
IPERF3_CPU_UTILIZATION_PERCENT_OUT_TAG = "cpu_utilization_percent"
IPERF3_END_RESULTS_TAG = "end"
TARGET_TAG = "target"
DELTA_PERCENTAGE_TAG = "delta_percentage"
THROUGHPUT_UNIT = "Mbps"
DURATION_UNIT = "seconds"
CPU_UTILIZATION_UNIT = "percentage"


CONFIG = {
    "time": 20,  # seconds
    "server_startup_time": 2,  # seconds
    "load_factor": 1,  # nr of iperf clients/vcpu
    "modes": {
        "g2h": [""],
        "h2g": ["-R"],
        "bd": ["", "-R"]
    },
    "protocols": [
        {
            "name": "tcp",
            "omit": 3,  # seconds
            # None == Default (128K)
            "payload_length": ["1024K", None, "1024"],
        }
    ],
    "hosts": {
        "instances": {
            "m5d.metal": {
                "cpus": [
                    {
                        "model": "Intel(R) Xeon(R) Platinum 8259CL CPU @ "
                        "2.50GHz",
                        "baseline_bw": {
                            "vmlinux-4.14.bin/ubuntu-18.04.ext4": {
                                "vsock-p1024K-1vcpu-g2h": {
                                    "target": 7001,
                                    "delta_percentage": 4
                                },
                                "vsock-pDEFAULT-1vcpu-g2h": {
                                    "target": 7036,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024-1vcpu-g2h": {
                                    "target": 2358,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024K-1vcpu-h2g": {
                                    "target": 5126,
                                    "delta_percentage": 5
                                },
                                "vsock-pDEFAULT-1vcpu-h2g": {
                                    "target": 5008,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024-1vcpu-h2g": {
                                    "target": 1854,
                                    "delta_percentage": 4
                                },
                                "vsock-p1024K-2vcpu-g2h": {
                                    "target": 7299,
                                    "delta_percentage": 6
                                },
                                "vsock-pDEFAULT-2vcpu-g2h": {
                                    "target": 7278,
                                    "delta_percentage": 7
                                },
                                "vsock-p1024-2vcpu-g2h": {
                                    "target": 3086,
                                    "delta_percentage": 6
                                },
                                "vsock-p1024K-2vcpu-h2g": {
                                    "target": 5933,
                                    "delta_percentage": 5
                                },
                                "vsock-pDEFAULT-2vcpu-h2g": {
                                    "target": 5845,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024-2vcpu-h2g": {
                                    "target": 2547,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024K-2vcpu-bd": {
                                    "target": 5761,
                                    "delta_percentage": 5
                                },
                                "vsock-pDEFAULT-2vcpu-bd": {
                                    "target": 5695,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024-2vcpu-bd": {
                                    "target": 2274,
                                    "delta_percentage": 5
                                }
                            }
                        },
                        "baseline_cpu_utilization": {
                            "vmlinux-4.14.bin/ubuntu-18.04.ext4": {
                                "vmm": {
                                    "vsock-p1024K-1vcpu-g2h": {
                                        "target": 51,
                                        "delta_percentage": 9
                                    },
                                    "vsock-pDEFAULT-1vcpu-g2h": {
                                        "target": 51,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024-1vcpu-g2h": {
                                        "target": 48,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024K-1vcpu-h2g": {
                                        "target": 60,
                                        "delta_percentage": 8
                                    },
                                    "vsock-pDEFAULT-1vcpu-h2g": {
                                        "target": 60,
                                        "delta_percentage": 8
                                    },
                                    "vsock-p1024-1vcpu-h2g": {
                                        "target": 43,
                                        "delta_percentage": 10
                                    },
                                    "vsock-p1024K-2vcpu-g2h": {
                                        "target": 65,
                                        "delta_percentage": 7
                                    },
                                    "vsock-pDEFAULT-2vcpu-g2h": {
                                        "target": 65,
                                        "delta_percentage": 8
                                    },
                                    "vsock-p1024-2vcpu-g2h": {
                                        "target": 70,
                                        "delta_percentage": 7
                                    },
                                    "vsock-p1024K-2vcpu-h2g": {
                                        "target": 77,
                                        "delta_percentage": 7
                                    },
                                    "vsock-pDEFAULT-2vcpu-h2g": {
                                        "target": 77,
                                        "delta_percentage": 8
                                    },
                                    "vsock-p1024-2vcpu-h2g": {
                                        "target": 64,
                                        "delta_percentage": 8
                                    },
                                    "vsock-p1024K-2vcpu-bd": {
                                        "target": 63,
                                        "delta_percentage": 8
                                    },
                                    "vsock-pDEFAULT-2vcpu-bd": {
                                        "target": 64,
                                        "delta_percentage": 8
                                    },
                                    "vsock-p1024-2vcpu-bd": {
                                        "target": 67,
                                        "delta_percentage": 8
                                    }
                                },
                                "vcpus_total": {
                                    "vsock-p1024K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "vsock-pDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-pDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "vsock-pDEFAULT-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024-2vcpu-g2h": {
                                        "target": 102,
                                        "delta_percentage": 10
                                    },
                                    "vsock-p1024K-2vcpu-h2g": {
                                        "target": 131,
                                        "delta_percentage": 6
                                    },
                                    "vsock-pDEFAULT-2vcpu-h2g": {
                                        "target": 131,
                                        "delta_percentage": 6
                                    },
                                    "vsock-p1024-2vcpu-h2g": {
                                        "target": 177,
                                        "delta_percentage": 6
                                    },
                                    "vsock-p1024K-2vcpu-bd": {
                                        "target": 121,
                                        "delta_percentage": 6
                                    },
                                    "vsock-pDEFAULT-2vcpu-bd": {
                                        "target": 122,
                                        "delta_percentage": 6
                                    },
                                    "vsock-p1024-2vcpu-bd": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    }
                                }
                            }
                        }
                    },
                    {
                        "model": "Intel(R) Xeon(R) Platinum 8175M CPU @ "
                        "2.50GHz",
                        "baseline_bw": {
                            "vmlinux-4.14.bin/ubuntu-18.04.ext4": {
                                "vsock-p1024K-1vcpu-g2h": {
                                    "target": 5934,
                                    "delta_percentage": 5
                                },
                                "vsock-pDEFAULT-1vcpu-g2h": {
                                    "target": 5944,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024-1vcpu-g2h": {
                                    "target": 946,
                                    "delta_percentage": 13
                                },
                                "vsock-p1024K-1vcpu-h2g": {
                                    "target": 3990,
                                    "delta_percentage": 4
                                },
                                "vsock-pDEFAULT-1vcpu-h2g": {
                                    "target": 3893,
                                    "delta_percentage": 4
                                },
                                "vsock-p1024-1vcpu-h2g": {
                                    "target": 301,
                                    "delta_percentage": 4
                                },
                                "vsock-p1024K-2vcpu-g2h": {
                                    "target": 6726,
                                    "delta_percentage": 6
                                },
                                "vsock-pDEFAULT-2vcpu-g2h": {
                                    "target": 6644,
                                    "delta_percentage": 9
                                },
                                "vsock-p1024-2vcpu-g2h": {
                                    "target": 2586,
                                    "delta_percentage": 6
                                },
                                "vsock-p1024K-2vcpu-h2g": {
                                    "target": 4837,
                                    "delta_percentage": 7
                                },
                                "vsock-pDEFAULT-2vcpu-h2g": {
                                    "target": 4746,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024-2vcpu-h2g": {
                                    "target": 1554,
                                    "delta_percentage": 9
                                },
                                "vsock-p1024K-2vcpu-bd": {
                                    "target": 4497,
                                    "delta_percentage": 5
                                },
                                "vsock-pDEFAULT-2vcpu-bd": {
                                    "target": 4473,
                                    "delta_percentage": 5
                                },
                                "vsock-p1024-2vcpu-bd": {
                                    "target": 1280,
                                    "delta_percentage": 10
                                }
                            }
                        },
                        "baseline_cpu_utilization": {
                            "vmlinux-4.14.bin/ubuntu-18.04.ext4": {
                                "vmm": {
                                    "vsock-p1024K-1vcpu-g2h": {
                                        "target": 51,
                                        "delta_percentage": 9
                                    },
                                    "vsock-pDEFAULT-1vcpu-g2h": {
                                        "target": 52,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024-1vcpu-g2h": {
                                        "target": 44,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024K-1vcpu-h2g": {
                                        "target": 66,
                                        "delta_percentage": 8
                                    },
                                    "vsock-pDEFAULT-1vcpu-h2g": {
                                        "target": 65,
                                        "delta_percentage": 8
                                    },
                                    "vsock-p1024-1vcpu-h2g": {
                                        "target": 32,
                                        "delta_percentage": 12
                                    },
                                    "vsock-p1024K-2vcpu-g2h": {
                                        "target": 63,
                                        "delta_percentage": 8
                                    },
                                    "vsock-pDEFAULT-2vcpu-g2h": {
                                        "target": 63,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024-2vcpu-g2h": {
                                        "target": 65,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024K-2vcpu-h2g": {
                                        "target": 79,
                                        "delta_percentage": 10
                                    },
                                    "vsock-pDEFAULT-2vcpu-h2g": {
                                        "target": 79,
                                        "delta_percentage": 8
                                    },
                                    "vsock-p1024-2vcpu-h2g": {
                                        "target": 60,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024K-2vcpu-bd": {
                                        "target": 67,
                                        "delta_percentage": 7
                                    },
                                    "vsock-pDEFAULT-2vcpu-bd": {
                                        "target": 68,
                                        "delta_percentage": 7
                                    },
                                    "vsock-p1024-2vcpu-bd": {
                                        "target": 65,
                                        "delta_percentage": 8
                                    }
                                },
                                "vcpus_total": {
                                    "vsock-p1024K-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-pDEFAULT-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024-1vcpu-g2h": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024K-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-pDEFAULT-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 6
                                    },
                                    "vsock-p1024-1vcpu-h2g": {
                                        "target": 99,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024K-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "vsock-pDEFAULT-2vcpu-g2h": {
                                        "target": 198,
                                        "delta_percentage": 5
                                    },
                                    "vsock-p1024-2vcpu-g2h": {
                                        "target": 119,
                                        "delta_percentage": 9
                                    },
                                    "vsock-p1024K-2vcpu-h2g": {
                                        "target": 124,
                                        "delta_percentage": 7
                                    },
                                    "vsock-pDEFAULT-2vcpu-h2g": {
                                        "target": 125,
                                        "delta_percentage": 6
                                    },
                                    "vsock-p1024-2vcpu-h2g": {
                                        "target": 181,
                                        "delta_percentage": 6
                                    },
                                    "vsock-p1024K-2vcpu-bd": {
                                        "target": 116,
                                        "delta_percentage": 7
                                    },
                                    "vsock-pDEFAULT-2vcpu-bd": {
                                        "target": 118,
                                        "delta_percentage": 7
                                    },
                                    "vsock-p1024-2vcpu-bd": {
                                        "target": 198,
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
