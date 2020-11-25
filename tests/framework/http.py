# Copyright 2018 Amazon.com, Inc. or its affiliates. All Rights Reserved.
# SPDX-License-Identifier: Apache-2.0
"""Wrapper over an http session with timed requests."""
# pylint: disable=unused-import
import requests
from requests_unixsocket import DEFAULT_SCHEME, UnixAdapter

from framework import decorators


class Session(requests.Session):
    """Wrapper over requests_unixsocket.Session limiting the call duration.

    Only the API calls relevant to Firecracker (GET, PUT, PATCH) are
    implemented.
    """

    def __init__(self):
        """Create a Session object and set the is_good_response callback."""
        super().__init__()

        # The `pool_connections` argument indicates the maximum number of
        # open connections allowed at a time. This value is set to 10 for
        # consistency with the micro-http's `MAX_CONNECTIONS`.
        self.mount(DEFAULT_SCHEME, UnixAdapter(pool_connections=10))

        def is_good_response(response: int):
            """Return `True` for all HTTP 2xx response codes."""
            return 200 <= response < 300

        def is_status_ok(response: int):
            """Return `True` when HTTP response code is 200 OK."""
            return response == 200

        def is_status_no_content(response: int):
            """Return `True` when HTTP response code is 204 NoContent."""
            return response == 204

        def is_status_bad_request(response: int):
            """Return `True` when HTTP response code is 400 BadRequest."""
            return response == 400

        def is_status_not_found(response: int):
            """Return `True` when HTTP response code is 404 NotFound."""
            return response == 404

        self.is_good_response = is_good_response
        self.is_status_ok = is_status_ok
        self.is_status_no_content = is_status_no_content
        self.is_status_bad_request = is_status_bad_request
        self.is_status_not_found = is_status_not_found

    @decorators.timed_request
    def get(self, url, **kwargs):
        """Wrap the GET call with duration limit."""
        # pylint: disable=method-hidden
        # The `untime` method overrides this, and pylint disapproves.
        return super().get(url, **kwargs)

    @decorators.timed_request
    def patch(self, url, data=None, **kwargs):
        """Wrap the PATCH call with duration limit."""
        # pylint: disable=method-hidden
        # The `untime` method overrides this, and pylint disapproves.
        return super().patch(url, data=data, **kwargs)

    @decorators.timed_request
    def put(self, url, data=None, **kwargs):
        """Wrap the PUT call with duration limit."""
        # pylint: disable=method-hidden
        # The `untime` method overrides this, and pylint disapproves.
        return super().put(url, data=data, **kwargs)

    def untime(self):
        """Restore the HTTP methods to their un-timed selves."""
        self.get = super().get
        self.patch = super().patch
        self.put = super().put
