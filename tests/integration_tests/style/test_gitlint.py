# Copyright 2020 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Tests ensuring desired style for commit messages."""

import os
import framework.utils as utils


def test_gitlint():
    """Fail if the default gitlint rules do not apply."""
    os.environ['LC_ALL'] = 'C.UTF-8'
    os.environ['LANG'] = 'C.UTF-8'
    try:
        utils.run_cmd('gitlint --commits origin/master..HEAD'
                      ' --extra-path framework/gitlint_rules.py')
    except ChildProcessError as error:
        assert False, "Commit message violates gitlint rules: {}".format(error)
