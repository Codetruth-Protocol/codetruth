/**
 * Sample TypeScript file for testing CTP analysis.
 *
 * This module demonstrates various code patterns that CTP should detect.
 */

interface PaymentRequest {
  amount: number;
  currency: string;
  customerId: string;
  idempotencyKey: string;
}

interface PaymentResult {
  status: 'success' | 'failed' | 'pending';
  transactionId?: string;
  error?: string;
}

/**
 * Calculate factorial of a number.
 */
export function factorial(n: number): number {
  if (n <= 1) return 1;
  return n * factorial(n - 1);
}

/**
 * Payment processor with retry logic and idempotency.
 */
export class PaymentProcessor {
  private apiKey: string;
  private maxRetries: number;

  constructor(apiKey: string, maxRetries = 3) {
    this.apiKey = apiKey;
    this.maxRetries = maxRetries;
  }

  /**
   * Process a payment with exponential backoff retry.
   *
   * @param request - Payment request details
   * @returns Payment result
   */
  async processPayment(request: PaymentRequest): Promise<PaymentResult> {
    if (request.amount <= 0) {
      throw new Error('Amount must be positive');
    }

    for (let attempt = 0; attempt < this.maxRetries; attempt++) {
      try {
        const result = await this.callPaymentApi(request);
        return result;
      } catch (error) {
        if (attempt === this.maxRetries - 1) {
          throw error;
        }
        // Exponential backoff with jitter
        const delay = Math.pow(2, attempt) * 1000 + Math.random() * 100;
        await this.sleep(delay);
      }
    }

    return { status: 'failed', error: 'Max retries exceeded' };
  }

  private async callPaymentApi(request: PaymentRequest): Promise<PaymentResult> {
    // This would call Stripe/etc in production
    const response = await fetch('https://api.stripe.com/v1/charges', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${this.apiKey}`,
        'Idempotency-Key': request.idempotencyKey,
      },
      body: JSON.stringify({
        amount: request.amount,
        currency: request.currency,
        customer: request.customerId,
      }),
    });

    if (!response.ok) {
      throw new Error(`Payment failed: ${response.status}`);
    }

    const data = await response.json();
    return { status: 'success', transactionId: data.id };
  }

  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Read configuration from a file.
 */
export async function readConfig(path: string): Promise<Record<string, unknown> | null> {
  try {
    const fs = await import('fs/promises');
    const content = await fs.readFile(path, 'utf-8');
    return JSON.parse(content);
  } catch {
    return null;
  }
}
