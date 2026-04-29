/**
 * MCP Client for CodeTruth VS Code Extension
 * 
 * Handles stdio communication with ctp-mcp server using JSON-RPC 2.0 protocol.
 */

import * as cp from 'child_process';
import { EventEmitter } from 'events';

interface MCPRequest {
  jsonrpc: '2.0';
  id: number | string;
  method: string;
  params?: any;
}

interface MCPResponse {
  jsonrpc: '2.0';
  id: number | string;
  result?: any;
  error?: {
    code: number;
    message: string;
    data?: any;
  };
}

interface MCPToolCall {
  name: string;
  arguments?: Record<string, any>;
}

export class MCPClient {
  private process: cp.ChildProcess | null = null;
  private requestId = 0;
  private pendingRequests = new Map<number | string, {
    resolve: (value: any) => void;
    reject: (error: Error) => void;
  }>();
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 3;
  private errorListeners: Array<(error: Error) => void> = [];
  private exitListeners: Array<(info: { code: number | null; signal: string | null }) => void> = [];

  constructor(private serverPath: string) {
    // No super() needed since we don't extend EventEmitter anymore
  }

  /**
   * Start the MCP server process
   */
  async start(): Promise<void> {
    if (this.process) {
      return;
    }

    return new Promise((resolve, reject) => {
      this.process = cp.spawn(this.serverPath, [], {
        stdio: ['pipe', 'pipe', 'pipe']
      });

      if (!this.process.stdin || !this.process.stdout) {
        reject(new Error('Failed to spawn MCP server process'));
        return;
      }

      this.process.on('error', (error) => {
        this.errorListeners.forEach(listener => listener(error));
        reject(error);
      });

      this.process.on('exit', (code, signal) => {
        this.exitListeners.forEach(listener => listener({ code, signal }));
        this.process = null;
        
        // Attempt to reconnect if not a clean shutdown
        if (code !== 0 && this.reconnectAttempts < this.maxReconnectAttempts) {
          this.reconnectAttempts++;
          setTimeout(() => this.start().catch(console.error), 1000);
        }
      });

      // Handle stdout messages
      let buffer = '';
      this.process.stdout.on('data', (data) => {
        buffer += data.toString();
        
        // Process complete JSON-RPC messages
        const lines = buffer.split('\n');
        buffer = lines.pop() || '';
        
        for (const line of lines) {
          if (line.trim()) {
            try {
              const response: MCPResponse = JSON.parse(line);
              this.handleResponse(response);
            } catch (e) {
              this.errorListeners.forEach(listener => listener(new Error(`Failed to parse MCP response: ${e}`)));
            }
          }
        }
      });

      // Send initialize request
      this.sendRequest('initialize', {
        protocolVersion: '2025-03-26',
        capabilities: {},
        clientInfo: {
          name: 'codetruth-vscode',
          version: '0.1.0'
        }
      }).then(() => {
        this.reconnectAttempts = 0;
        resolve();
      }).catch(reject);
    });
  }

  /**
   * Stop the MCP server process
   */
  async stop(): Promise<void> {
    if (this.process) {
      this.process.kill('SIGTERM');
      this.process = null;
    }
    
    // Reject all pending requests
    for (const [id, { reject }] of this.pendingRequests) {
      reject(new Error('MCP client stopped'));
    }
    this.pendingRequests.clear();
  }

  /**
   * Call an MCP tool
   */
  async callTool(toolName: string, args: Record<string, any> = {}): Promise<any> {
    return this.sendRequest('tools/call', {
      name: toolName,
      arguments: args
    });
  }

  /**
   * List available tools
   */
  async listTools(): Promise<any> {
    return this.sendRequest('tools/list', {});
  }

  /**
   * Send a JSON-RPC request
   */
  private sendRequest(method: string, params: any = {}): Promise<any> {
    return new Promise((resolve, reject) => {
      const id = ++this.requestId;
      
      const request: MCPRequest = {
        jsonrpc: '2.0',
        id,
        method,
        params
      };

      this.pendingRequests.set(id, { resolve, reject });

      if (!this.process || !this.process.stdin) {
        reject(new Error('MCP server not running'));
        return;
      }

      this.process.stdin.write(JSON.stringify(request) + '\n', (error) => {
        if (error) {
          this.pendingRequests.delete(id);
          reject(error);
        }
      });

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error('MCP request timeout'));
        }
      }, 30000);
    });
  }

  /**
   * Handle a JSON-RPC response
   */
  private handleResponse(response: MCPResponse): void {
    const { id, result, error } = response;
    
    const pending = this.pendingRequests.get(id);
    if (!pending) {
      return;
    }

    this.pendingRequests.delete(id);

    if (error) {
      pending.reject(new Error(`MCP error ${error.code}: ${error.message}`));
    } else {
      pending.resolve(result);
    }
  }

  /**
   * Check if the server is running
   */
  isRunning(): boolean {
    return this.process !== null && !this.process.killed;
  }

  /**
   * Register event listeners
   */
  on(event: 'error', listener: (error: Error) => void): void;
  on(event: 'exit', listener: (info: { code: number | null; signal: string | null }) => void): void;
  on(event: string, listener: (...args: any[]) => void): void {
    if (event === 'error') {
      this.errorListeners.push(listener as (error: Error) => void);
    } else if (event === 'exit') {
      this.exitListeners.push(listener as (info: { code: number | null; signal: string | null }) => void);
    }
  }
}
