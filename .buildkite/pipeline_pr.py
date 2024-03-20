#!/usr/bin/env python3
# Copyright 2022 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0

"""Generate Buildkite pipelines dynamically"""

from common import (
    COMMON_PARSER,
    devtool_test,
    get_changed_files,
    group,
    overlay_dict,
    pipeline_to_json,
    run_all_tests,
)

# Buildkite default job priority is 0. Setting this to 1 prioritizes PRs over
# scheduled jobs and other batch jobs.
DEFAULT_PRIORITY = 1


args = COMMON_PARSER.parse_args()

step_style = {
    "command": "./tools/devtool -y checkstyle",
    "label": "🪶 Style",
    "priority": DEFAULT_PRIORITY,
}

defaults = {
    "instances": args.instances,
    "platforms": args.platforms,
    # buildkite step parameters
    "priority": DEFAULT_PRIORITY,
    "timeout_in_minutes": 45,
    "artifacts": ["./test_results/**/*"],
}
defaults = overlay_dict(defaults, args.step_param)

defaults_once_per_architecture = defaults.copy()
defaults_once_per_architecture["instances"] = ["m6i.metal", "m7g.metal"]
defaults_once_per_architecture["platforms"] = [("al2", "linux_5.10")]


devctr_grp = group(
    "🐋 Dev Container Sanity Build",
    "./tools/devtool -y build_devctr",
    **defaults_once_per_architecture,
)

release_grp = group(
    "📦 Release Sanity Build",
    "./tools/devtool -y make_release",
    **defaults_once_per_architecture,
)

build_grp = group(
    "📦 Build",
    "./tools/devtool -y test -- ../tests/integration_tests/build/",
    **defaults,
)

functional_grp = group(
    "⚙ Functional and security 🔒",
    devtool_test(
        pytest_opts="-n 8 --dist worksteal integration_tests/{{functional,security}}",
        binary_dir=args.binary_dir,
    ),
    **defaults,
)

defaults_for_performance = overlay_dict(
    defaults,
    {
        # We specify higher priority so the ag=1 jobs get picked up before the ag=n
        # jobs in ag=1 agents
        "priority": DEFAULT_PRIORITY + 1,
        "agents": {"ag": 1},
    },
)

performance_grp = group(
    "⏱ Performance",
    devtool_test(
        devtool_opts="--performance -c 1-10 -m 0",
        pytest_opts="../tests/integration_tests/performance/",
        binary_dir=args.binary_dir,
    ),
    **defaults_for_performance,
)

defaults_for_kani = overlay_dict(
    defaults_for_performance,
    {
        # Kani runs fastest on m6i.metal
        "instances": ["m6a.metal"],
        "platforms": [("al2", "linux_5.10")],
        "timeout_in_minutes": 300,
    },
)

kani_grp = group(
    "🔍 Kani",
    "./tools/devtool -y test -- ../tests/integration_tests/test_kani.py -n auto",
    **defaults_for_kani,
)
for step in kani_grp["steps"]:
    step["label"] = "🔍 Kani"

steps = [step_style]
changed_files = get_changed_files()

# run sanity build of devtool if Dockerfile is changed
if any(x.name == "Dockerfile" for x in changed_files):
    steps.append(devctr_grp)

if any(
    x.parent.name == "tools" and ("release" in x.name or x.name == "devtool")
    for x in changed_files
):
    steps.append(release_grp)

if not changed_files or any(
    x.suffix in [".rs", ".toml", ".lock"] for x in changed_files
):
    steps.append(kani_grp)

if run_all_tests(changed_files):
    steps += [
        build_grp,
        functional_grp,
        performance_grp,
    ]

pipeline = {"steps": steps}
print(pipeline_to_json(pipeline))
