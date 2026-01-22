"""
CodeTruth Protocol - Python SDK

Provides Python bindings for the CTP analysis engine.
"""

from .types import (
    ExplanationGraph,
    MinimalAnalysis,
    DriftSeverity,
    Intent,
    Behavior,
    DriftAnalysis,
)
from .analyzer import CTPAnalyzer
from .client import CTPClient

__version__ = "0.1.0"
CTP_VERSION = "1.0.0"

__all__ = [
    "CTPAnalyzer",
    "CTPClient",
    "ExplanationGraph",
    "MinimalAnalysis",
    "DriftSeverity",
    "Intent",
    "Behavior",
    "DriftAnalysis",
    "__version__",
    "CTP_VERSION",
]
