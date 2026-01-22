"""
Sample Python file for testing CTP analysis.

This module demonstrates various code patterns that CTP should detect.
"""

import os
from typing import Optional


def factorial(n: int) -> int:
    """Calculate the factorial of a number using recursion."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)


class PaymentProcessor:
    """Process payment transactions with retry logic."""

    def __init__(self, api_key: str):
        self.api_key = api_key
        self.max_retries = 3

    def process_payment(
        self,
        amount: float,
        currency: str,
        idempotency_key: str,
    ) -> dict:
        """
        Process a payment with idempotency safeguards.

        Args:
            amount: Payment amount
            currency: Currency code (e.g., 'USD')
            idempotency_key: Unique key to prevent duplicate charges

        Returns:
            Payment result dictionary
        """
        # Validate inputs
        if amount <= 0:
            raise ValueError("Amount must be positive")

        # Process with retry
        for attempt in range(self.max_retries):
            try:
                result = self._call_payment_api(amount, currency, idempotency_key)
                return result
            except Exception as e:
                if attempt == self.max_retries - 1:
                    raise
                # Exponential backoff
                import time
                time.sleep(2 ** attempt)

        return {"status": "failed"}

    def _call_payment_api(
        self,
        amount: float,
        currency: str,
        idempotency_key: str,
    ) -> dict:
        """Call external payment API."""
        # This would call Stripe/etc in production
        return {"status": "success", "transaction_id": "txn_123"}


def read_config(path: str) -> Optional[dict]:
    """Read configuration from file."""
    if not os.path.exists(path):
        return None

    with open(path, 'r') as f:
        import json
        return json.load(f)
