/**
 * CTP API Client for communicating with CodeTruth servers
 */

import type { ExplanationGraph, MinimalAnalysis } from './types';

export interface CTPClientConfig {
  baseUrl?: string;
  apiKey?: string;
  timeout?: number;
}

export class CTPClient {
  private baseUrl: string;
  private apiKey?: string;
  private timeout: number;

  constructor(config: CTPClientConfig = {}) {
    this.baseUrl = config.baseUrl || 'http://localhost:9999';
    this.apiKey = config.apiKey;
    this.timeout = config.timeout || 30000;
  }

  async analyze(filePath: string): Promise<ExplanationGraph> {
    const response = await this.request('/analyze', {
      method: 'POST',
      body: JSON.stringify({ file_path: filePath }),
    });
    return response.json();
  }

  async analyzeCode(code: string, language: string): Promise<ExplanationGraph> {
    const response = await this.request('/analyze/code', {
      method: 'POST',
      body: JSON.stringify({ code, language }),
    });
    return response.json();
  }

  async analyzeMinimal(filePath: string): Promise<MinimalAnalysis> {
    const response = await this.request('/analyze/minimal', {
      method: 'POST',
      body: JSON.stringify({ file_path: filePath }),
    });
    return response.json();
  }

  async checkPolicies(filePath: string, policies?: string[]): Promise<ExplanationGraph> {
    const response = await this.request('/check', {
      method: 'POST',
      body: JSON.stringify({ file_path: filePath, policies }),
    });
    return response.json();
  }

  private async request(path: string, options: RequestInit): Promise<Response> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    };

    if (this.apiKey) {
      headers['Authorization'] = `Bearer ${this.apiKey}`;
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${path}`, {
        ...options,
        headers: { ...headers, ...options.headers },
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new CTPError(`Request failed: ${response.status}`, response.status);
      }

      return response;
    } finally {
      clearTimeout(timeoutId);
    }
  }
}

export class CTPError extends Error {
  constructor(message: string, public statusCode?: number) {
    super(message);
    this.name = 'CTPError';
  }
}
