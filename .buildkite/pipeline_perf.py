#!/usr/bin/env python3
# Copyright 2023 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

"""Generate Buildkite performance pipelines dynamically"""

from common import COMMON_PARSER, devtool_test, group, overlay_dict, pipeline_to_json

perf_test = {
    "block": {
        "label": "🖴 Block Performance",
        "test_path": "integration_tests/performance/test_block_performance.py",
        "devtool_opts": "-c 1-10 -m 0",
        "timeout_in_minutes": 240,
    },
    "snapshot-latency": {
        "label": "📸 Snapshot Latency",
        "test_path": "integration_tests/performance/test_snapshot_restore_performance.py",
        "devtool_opts": "-c 1-12 -m 0",
        "timeout_in_minutes": 60,
    },
    "vsock-throughput": {
        "label": "🧦 Vsock Throughput",
        "test_path": "integration_tests/performance/test_vsock_throughput.py",
        "devtool_opts": "-c 1-10 -m 0",
        "timeout_in_minutes": 20,
    },
    "network-latency": {
        "label": "🖧 Network Latency",
        "test_path": "integration_tests/performance/test_network_latency.py",
        "devtool_opts": "-c 1-10 -m 0",
        "timeout_in_minutes": 10,
    },
    "network-throughput": {
        "label": "🖧 Network TCP Throughput",
        "test_path": "integration_tests/performance/test_network_tcp_throughput.py",
        "devtool_opts": "-c 1-10 -m 0",
        "timeout_in_minutes": 45,
    },
}


def build_group(test):
    """Build a Buildkite pipeline `group` step"""
    devtool_opts = test.pop("devtool_opts")
    test_path = test.pop("test_path")
    retries = test.pop("retries")
    binary_dir = test.pop("binary_dir")
    pytest_opts = f"-m nonci --reruns {retries} --perf-fail"
    return group(
        label=test.pop("label"),
        command=devtool_test(test_path, devtool_opts, pytest_opts, binary_dir),
        artifacts=["./test_results/*"],
        instances=test.pop("instances"),
        platforms=test.pop("platforms"),
        # and the rest can be command arguments
        **test,
    )


parser = COMMON_PARSER
parser.add_argument(
    "--test",
    choices=list(perf_test.keys()),
    required=False,
    help="performance test",
    action="append",
)
parser.add_argument("--retries", type=int, default=0)
args = parser.parse_args()
group_steps = []
tests = [perf_test[test] for test in args.test or perf_test.keys()]
for test_data in tests:
    test_data.setdefault("platforms", args.platforms)
    test_data.setdefault("instances", args.instances)
    # use ag=1 instances to make sure no two performance tests are scheduled on the same instance
    test_data.setdefault("agents", {"ag": 1})
    test_data["retries"] = args.retries
    test_data["timeout_in_minutes"] *= args.retries + 1
    test_data["binary_dir"] = args.binary_dir
    test_data = overlay_dict(test_data, args.step_param)
    test_data["retry"] = {
        "automatic": [
            # Agent was lost, retry one time
            # this can happen if we terminate the instance or the agent gets
            # disconnected for whatever reason
            {"exit_status": -1, "limit": 1},
        ]
    }
    group_steps.append(build_group(test_data))

pipeline = {
    "env": {},
    "steps": group_steps,
}
print(pipeline_to_json(pipeline))
