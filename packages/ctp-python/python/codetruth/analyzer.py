"""
CTP Analyzer - Pure Python implementation for analysis.
"""

import hashlib
import re
from pathlib import Path
from typing import Optional

from .types import (
    MinimalAnalysis,
    ExplanationGraph,
    DriftSeverity,
    Intent,
    Behavior,
    DriftAnalysis,
    Module,
    Metadata,
    SideEffect,
)


class CTPAnalyzer:
    """CodeTruth Protocol analyzer."""

    def __init__(self, use_rust_core: bool = True):
        """
        Initialize the analyzer.

        Args:
            use_rust_core: Whether to use the Rust core via PyO3 (faster).
                          Falls back to pure Python if not available.
        """
        self._rust_core = None
        if use_rust_core:
            try:
                from codetruth._core import RustAnalyzer
                self._rust_core = RustAnalyzer()
            except ImportError:
                pass

    def analyze_file(self, path: str | Path) -> ExplanationGraph:
        """Analyze a file and return full explanation graph."""
        path = Path(path)
        
        # Use Rust core if available for better performance
        if self._rust_core is not None:
            try:
                result = self._rust_core.analyze_file(str(path))
                return self._dict_to_explanation_graph(result)
            except Exception:
                pass  # Fall back to pure Python
        
        content = path.read_text(encoding="utf-8")
        language = self._detect_language(path)
        return self.analyze_code(content, language, path.name)
    
    def _dict_to_explanation_graph(self, data: dict) -> ExplanationGraph:
        """Convert Rust analysis dict to ExplanationGraph."""
        from .types import PolicyResults, History, Evolution
        
        module = Module(
            name=data["module"]["name"],
            path=data["module"]["path"],
            language=data["module"]["language"],
            lines_of_code=data["module"]["lines_of_code"],
            complexity_score=data["module"]["complexity_score"],
        )
        
        intent = Intent(
            declared_intent=data["intent"]["declared_intent"],
            inferred_intent=data["intent"]["inferred_intent"],
            confidence=data["intent"]["confidence"],
        )
        
        side_effects = [
            SideEffect(
                effect_type=se.get("effect_type", ""),
                description=se.get("description", ""),
                risk_level=se.get("risk_level", "MEDIUM"),
            )
            for se in data["behavior"].get("side_effects", [])
        ]
        
        behavior = Behavior(
            actual_behavior=data["behavior"]["actual_behavior"],
            side_effects=side_effects,
        )
        
        drift_severity = DriftSeverity[data["drift"]["drift_severity"]]
        drift = DriftAnalysis(
            drift_detected=data["drift"]["drift_detected"],
            drift_severity=drift_severity,
            drift_details=[],
        )
        
        return ExplanationGraph(
            ctp_version=data["ctp_version"],
            explanation_id=data["explanation_id"],
            module=module,
            intent=intent,
            behavior=behavior,
            drift=drift,
            metadata=Metadata(generator_version="0.1.0"),
        )

    def analyze_code(
        self, code: str, language: str, name: str = "unknown"
    ) -> ExplanationGraph:
        """Analyze code string and return full explanation graph."""
        content_hash = hashlib.sha256(code.encode()).hexdigest()

        module = Module(
            name=name,
            path="",
            language=language,
            lines_of_code=len(code.splitlines()),
            complexity_score=0.0,
        )

        declared_intent = self._extract_intent(code, language)
        behavior = self._analyze_behavior(code, language)
        inferred_intent = declared_intent or self._infer_intent(code)

        intent = Intent(
            declared_intent=declared_intent,
            inferred_intent=inferred_intent,
            confidence=0.8 if declared_intent else 0.5,
        )

        drift = self._detect_drift(intent, behavior)

        return ExplanationGraph(
            ctp_version="1.0.0",
            explanation_id=f"sha256:{content_hash}",
            module=module,
            intent=intent,
            behavior=behavior,
            drift=drift,
            metadata=Metadata(generator_version="0.1.0"),
        )

    def analyze_minimal(self, path: str | Path) -> MinimalAnalysis:
        """Analyze a file and return minimal analysis."""
        path = Path(path)
        content = path.read_text(encoding="utf-8")
        language = self._detect_language(path)

        content_hash = hashlib.sha256(content.encode()).hexdigest()
        intent = self._extract_intent(content, language)
        behavior = self._analyze_behavior(content, language)
        drift = self._detect_drift_severity(intent, behavior.actual_behavior)

        return MinimalAnalysis(
            ctp_version="1.0.0",
            file_hash=f"sha256:{content_hash}",
            intent=intent or "No declared intent",
            behavior=behavior.actual_behavior,
            drift=drift,
            confidence=0.8 if intent else 0.5,
        )

    def _detect_language(self, path: Path) -> str:
        """Detect programming language from file extension."""
        ext_map = {
            ".py": "python",
            ".js": "javascript",
            ".mjs": "javascript",
            ".ts": "typescript",
            ".tsx": "typescript",
            ".rs": "rust",
            ".go": "go",
            ".java": "java",
            ".rb": "ruby",
            ".php": "php",
        }
        return ext_map.get(path.suffix.lower(), "unknown")

    def _extract_intent(self, code: str, language: str) -> str:
        """Extract declared intent from comments/docstrings."""
        patterns = {
            "python": [r'"""([\s\S]*?)"""', r"'''([\s\S]*?)'''"],
            "javascript": [r"/\*\*([\s\S]*?)\*/"],
            "typescript": [r"/\*\*([\s\S]*?)\*/"],
            "rust": [r"///(.+)", r"//!(.+)"],
        }

        lang_patterns = patterns.get(language, patterns["javascript"])

        for pattern in lang_patterns:
            match = re.search(pattern, code)
            if match:
                text = match.group(1)
                text = re.sub(r"^\s*\*\s?", "", text, flags=re.MULTILINE)
                return text.strip()[:280]

        return ""

    def _analyze_behavior(self, code: str, language: str) -> Behavior:
        """Analyze actual behavior of the code."""
        lines = code.splitlines()
        parts = []
        side_effects = []

        func_patterns = {
            "python": r"^\s*(async\s+)?def\s+",
            "javascript": r"^\s*(async\s+)?function\s+|^\s*const\s+\w+\s*=",
            "typescript": r"^\s*(async\s+)?function\s+|^\s*const\s+\w+\s*=",
            "rust": r"^\s*(pub\s+)?(async\s+)?fn\s+",
        }

        pattern = func_patterns.get(language, func_patterns["javascript"])
        func_count = sum(1 for line in lines if re.match(pattern, line))

        if func_count > 0:
            parts.append(f"{func_count} function(s)")

        io_patterns = ["open(", "read(", "write(", "fetch(", "fs."]
        if any(p in code for p in io_patterns):
            parts.append("file/network I/O")
            side_effects.append(SideEffect("io", "File/network operations", "MEDIUM"))

        db_patterns = ["SELECT ", "INSERT ", "UPDATE ", "DELETE ", "mongodb", "prisma"]
        if any(p.lower() in code.lower() for p in db_patterns):
            parts.append("database operations")
            side_effects.append(SideEffect("database", "Database operations", "HIGH"))

        actual = f"Performs {', '.join(parts)}" if parts else "Simple logic"

        return Behavior(actual_behavior=actual, side_effects=side_effects)

    def _infer_intent(self, code: str) -> str:
        """Infer intent from code patterns."""
        code_lower = code.lower()

        if "test" in code_lower or "assert" in code_lower:
            return "Test code for verifying functionality"
        if "auth" in code_lower or "login" in code_lower:
            return "Authentication/authorization logic"
        if "payment" in code_lower or "charge" in code_lower:
            return "Payment processing logic"
        if "api" in code_lower or "endpoint" in code_lower:
            return "API endpoint handler"

        return "General purpose code module"

    def _detect_drift(self, intent: Intent, behavior: Behavior) -> DriftAnalysis:
        """Detect drift between intent and behavior."""
        severity = self._detect_drift_severity(
            intent.declared_intent, behavior.actual_behavior
        )

        return DriftAnalysis(
            drift_detected=severity != DriftSeverity.NONE,
            drift_severity=severity,
            drift_details=[],
        )

    def _detect_drift_severity(self, intent: str, behavior: str) -> DriftSeverity:
        """Calculate drift severity based on similarity."""
        if not intent:
            return DriftSeverity.LOW

        intent_words = set(intent.lower().split())
        behavior_words = set(behavior.lower().split())

        if not intent_words or not behavior_words:
            return DriftSeverity.MEDIUM

        intersection = len(intent_words & behavior_words)
        union = len(intent_words | behavior_words)
        similarity = intersection / union if union > 0 else 0

        if similarity >= 0.7:
            return DriftSeverity.NONE
        if similarity >= 0.5:
            return DriftSeverity.LOW
        if similarity >= 0.3:
            return DriftSeverity.MEDIUM
        return DriftSeverity.HIGH
