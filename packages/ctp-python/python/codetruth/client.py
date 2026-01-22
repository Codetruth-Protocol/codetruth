"""
CTP API Client for communicating with CodeTruth servers.
"""

import json
from typing import Optional
from urllib.request import Request, urlopen
from urllib.error import HTTPError

from .types import ExplanationGraph, MinimalAnalysis


class CTPClient:
    """Client for communicating with CTP servers."""

    def __init__(
        self,
        base_url: str = "http://localhost:9999",
        api_key: Optional[str] = None,
        timeout: int = 30,
    ):
        self.base_url = base_url.rstrip("/")
        self.api_key = api_key
        self.timeout = timeout

    def analyze(self, file_path: str) -> dict:
        """Analyze a file via the CTP server."""
        return self._request("POST", "/analyze", {"file_path": file_path})

    def analyze_code(self, code: str, language: str) -> dict:
        """Analyze code string via the CTP server."""
        return self._request("POST", "/analyze/code", {"code": code, "language": language})

    def check_policies(self, file_path: str, policies: Optional[list[str]] = None) -> dict:
        """Check policy compliance via the CTP server."""
        return self._request("POST", "/check", {"file_path": file_path, "policies": policies})

    def _request(self, method: str, path: str, data: Optional[dict] = None) -> dict:
        """Make an HTTP request to the CTP server."""
        url = f"{self.base_url}{path}"
        headers = {"Content-Type": "application/json"}

        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"

        body = json.dumps(data).encode() if data else None
        request = Request(url, data=body, headers=headers, method=method)

        try:
            with urlopen(request, timeout=self.timeout) as response:
                return json.loads(response.read().decode())
        except HTTPError as e:
            raise CTPClientError(f"Request failed: {e.code} {e.reason}") from e


class CTPClientError(Exception):
    """Error from CTP client operations."""
    pass
