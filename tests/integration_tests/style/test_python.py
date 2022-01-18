# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests ensuring codebase style compliance for Python."""

from framework import utils


def test_python_style():
    """
    Test that python code passes style checks.

    @type: style
    """
    # List of linter commands that should be executed for each file
    linter_cmds = [
        # Pylint
        'python3 -m pylint --jobs=0 --persistent=no --score=no ' \
        '--output-format=colorized --attr-rgx="[a-z_][a-z0-9_]{1,30}$" ' \
        '--argument-rgx="[a-z_][a-z0-9_]{1,35}$" ' \
        '--variable-rgx="[a-z_][a-z0-9_]{1,30}$" --disable=' \
        'bad-continuation,fixme,too-many-instance-attributes,import-error,' \
        'too-many-locals,too-many-arguments,consider-using-f-string,' \
        'consider-using-with',

        # pycodestyle
        'python3 -m pycodestyle --show-pep8 --show-source --exclude=../build',

        # pydocstyle
        "python3 -m pydocstyle --explain --source"]

    # Get all *.py files from the project
    python_files = utils.get_files_from(
        find_path="..",
        pattern="*.py",
        exclude_names=["build", ".kernel"])

    # Assert if somehow no python files were found
    assert len(python_files) != 0

    # Run commands
    utils.run_cmd_list_async([
        f"{cmd} {fname}" for cmd in linter_cmds for fname in python_files
    ])
