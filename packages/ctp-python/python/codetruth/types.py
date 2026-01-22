"""
Core CTP types matching the protocol specification.
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


class DriftSeverity(str, Enum):
    NONE = "NONE"
    LOW = "LOW"
    MEDIUM = "MEDIUM"
    HIGH = "HIGH"
    CRITICAL = "CRITICAL"


class DriftType(str, Enum):
    INTENT = "INTENT"
    POLICY = "POLICY"
    ASSUMPTION = "ASSUMPTION"
    IMPLEMENTATION = "IMPLEMENTATION"


class PolicyStatus(str, Enum):
    PASS = "PASS"
    FAIL = "FAIL"
    WARNING = "WARNING"
    SKIP = "SKIP"


@dataclass
class Intent:
    declared_intent: str = ""
    inferred_intent: str = ""
    confidence: float = 0.0
    business_context: str = ""
    technical_rationale: str = ""


@dataclass
class SideEffect:
    effect_type: str = ""
    description: str = ""
    risk_level: str = "LOW"


@dataclass
class Behavior:
    actual_behavior: str = ""
    entry_points: list = field(default_factory=list)
    exit_points: list = field(default_factory=list)
    side_effects: list[SideEffect] = field(default_factory=list)
    dependencies: list = field(default_factory=list)


@dataclass
class DriftDetail:
    drift_type: DriftType = DriftType.INTENT
    expected: str = ""
    actual: str = ""
    file: str = ""
    line_start: int = 0
    line_end: int = 0
    remediation: str = ""


@dataclass
class DriftAnalysis:
    drift_detected: bool = False
    drift_severity: DriftSeverity = DriftSeverity.NONE
    drift_details: list[DriftDetail] = field(default_factory=list)


@dataclass
class Module:
    name: str = ""
    path: str = ""
    language: str = ""
    lines_of_code: int = 0
    complexity_score: float = 0.0


@dataclass
class Metadata:
    generated_at: str = ""
    generator_name: str = "CodeTruth"
    generator_version: str = ""
    llm_provider: Optional[str] = None
    llm_model: Optional[str] = None


@dataclass
class ExplanationGraph:
    ctp_version: str = "1.0.0"
    explanation_id: str = ""
    module: Module = field(default_factory=Module)
    intent: Intent = field(default_factory=Intent)
    behavior: Behavior = field(default_factory=Behavior)
    drift: DriftAnalysis = field(default_factory=DriftAnalysis)
    metadata: Metadata = field(default_factory=Metadata)


@dataclass
class MinimalAnalysis:
    ctp_version: str = "1.0.0"
    file_hash: str = ""
    intent: str = ""
    behavior: str = ""
    drift: DriftSeverity = DriftSeverity.NONE
    confidence: float = 0.0
