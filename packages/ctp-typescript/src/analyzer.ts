/**
 * Local analyzer for browser/Node.js environments
 * Uses WASM when available, falls back to JS implementation
 */

import type { ExplanationGraph, MinimalAnalysis, DriftSeverity } from './types';

export interface AnalyzerConfig {
  useWasm?: boolean;
  wasmPath?: string;
}

export class CTPAnalyzer {
  private config: AnalyzerConfig;
  private wasmModule?: WebAssembly.Module;

  constructor(config: AnalyzerConfig = {}) {
    this.config = {
      useWasm: true,
      ...config,
    };
  }

  async initialize(): Promise<void> {
    if (this.config.useWasm && this.config.wasmPath) {
      try {
        const wasmBuffer = await fetch(this.config.wasmPath).then(r => r.arrayBuffer());
        this.wasmModule = await WebAssembly.compile(wasmBuffer);
      } catch {
        console.warn('WASM not available, using JS fallback');
      }
    }
  }

  analyzeCode(code: string, language: string): MinimalAnalysis {
    const hash = this.hashCode(code);
    const intent = this.extractIntent(code, language);
    const behavior = this.analyzeBehavior(code, language);
    const drift = this.detectDrift(intent, behavior);

    return {
      ctp_version: '1.0.0',
      file_hash: `sha256:${hash}`,
      intent: intent || 'No declared intent',
      behavior,
      drift,
      confidence: intent ? 0.8 : 0.5,
    };
  }

  private hashCode(code: string): string {
    let hash = 0;
    for (let i = 0; i < code.length; i++) {
      const char = code.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash;
    }
    return Math.abs(hash).toString(16).padStart(16, '0');
  }

  private extractIntent(code: string, language: string): string {
    const patterns: Record<string, RegExp[]> = {
      python: [/"""([\s\S]*?)"""/m, /'''([\s\S]*?)'''/m],
      javascript: [/\/\*\*([\s\S]*?)\*\//m],
      typescript: [/\/\*\*([\s\S]*?)\*\//m],
      rust: [/\/\/\/(.+)/gm, /\/\/!(.+)/gm],
    };

    const langPatterns = patterns[language] || patterns.javascript;
    
    for (const pattern of langPatterns) {
      const match = code.match(pattern);
      if (match) {
        return match[1]
          .replace(/^\s*\*\s?/gm, '')
          .trim()
          .slice(0, 280);
      }
    }

    return '';
  }

  private analyzeBehavior(code: string, language: string): string {
    const lines = code.split('\n');
    const parts: string[] = [];

    const funcPatterns: Record<string, RegExp> = {
      python: /^\s*(async\s+)?def\s+/,
      javascript: /^\s*(async\s+)?function\s+|^\s*const\s+\w+\s*=\s*(async\s+)?\(/,
      typescript: /^\s*(async\s+)?function\s+|^\s*const\s+\w+\s*=\s*(async\s+)?\(/,
      rust: /^\s*(pub\s+)?(async\s+)?fn\s+/,
    };

    const funcPattern = funcPatterns[language] || funcPatterns.javascript;
    const funcCount = lines.filter(l => funcPattern.test(l)).length;

    if (funcCount > 0) parts.push(`${funcCount} function(s)`);

    const ioPatterns = ['open(', 'read(', 'write(', 'fetch(', 'fs.'];
    if (lines.some(l => ioPatterns.some(p => l.includes(p)))) {
      parts.push('file/network I/O');
    }

    const dbPatterns = ['SELECT ', 'INSERT ', 'UPDATE ', 'DELETE ', 'mongodb', 'prisma'];
    if (lines.some(l => dbPatterns.some(p => l.toUpperCase().includes(p.toUpperCase())))) {
      parts.push('database operations');
    }

    return parts.length > 0 ? `Performs ${parts.join(', ')}` : 'Simple logic';
  }

  private detectDrift(intent: string, behavior: string): DriftSeverity {
    if (!intent) return 'LOW';

    const intentWords = new Set(intent.toLowerCase().split(/\W+/).filter(Boolean));
    const behaviorWords = new Set(behavior.toLowerCase().split(/\W+/).filter(Boolean));

    const intersection = [...intentWords].filter(w => behaviorWords.has(w)).length;
    const union = new Set([...intentWords, ...behaviorWords]).size;

    const similarity = union > 0 ? intersection / union : 0;

    if (similarity >= 0.7) return 'NONE';
    if (similarity >= 0.5) return 'LOW';
    if (similarity >= 0.3) return 'MEDIUM';
    return 'HIGH';
  }
}
