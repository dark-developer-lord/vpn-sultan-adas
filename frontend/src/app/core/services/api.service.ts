import { Injectable } from '@angular/core';
import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Observable, throwError } from 'rxjs';
import { catchError } from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class ApiService {
  private baseUrl = this.getBaseUrl();

  constructor(private http: HttpClient) {}

  private getBaseUrl(): string {
    // Use production URL on VPS, localhost for development
    if (typeof window !== 'undefined') {
      const hostname = window.location.hostname;
      if (hostname === 'localhost' || hostname === '127.0.0.1') {
        return 'http://localhost:3000';
      }
      return `http://${hostname}:3000`;
    }
    return 'http://localhost:3000';
  }

  // Auth endpoints
  register(email: string, password: string): Observable<any> {
    return this.http.post<any>(`${this.baseUrl}/auth/register`, { email, password }, {
      headers: { 'Content-Type': 'application/json' }
    }).pipe(catchError(this.handleError));
  }

  login(email: string, password: string): Observable<any> {
    return this.http.post<any>(`${this.baseUrl}/auth/login`, { email, password }, {
      headers: { 'Content-Type': 'application/json' }
    }).pipe(catchError(this.handleError));
  }

  // Removed logout - not implemented in backend yet

  // Health check
  getHealth(): Observable<any> {
    return this.http.get(`${this.baseUrl}/health`)
      .pipe(catchError(this.handleError));
  }

  // Peers (token added automatically by interceptor)
  getPeers(): Observable<any> {
    return this.http.get(`${this.baseUrl}/peers`)
      .pipe(catchError(this.handleError));
  }

  createPeer(nodeId: string, name: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/peers`, { node_id: nodeId, name })
      .pipe(catchError(this.handleError));
  }

  getPeer(peerId: string): Observable<any> {
    return this.http.get(`${this.baseUrl}/peers/${peerId}`)
      .pipe(catchError(this.handleError));
  }

  getPeerConfig(peerId: string): Observable<any> {
    return this.http.get(`${this.baseUrl}/peers/${peerId}/config`)
      .pipe(catchError(this.handleError));
  }

  revokePeer(peerId: string): Observable<any> {
    return this.http.delete(`${this.baseUrl}/peers/${peerId}`)
      .pipe(catchError(this.handleError));
  }

  // Nodes
  getNodes(): Observable<any> {
    return this.http.get(`${this.baseUrl}/nodes`)
      .pipe(catchError(this.handleError));
  }

  // Agent registration
  registerAgent(nodeName: string, publicIp: string, internalIp: string, wgPublicKey: string): Observable<any> {
    return this.http.post(`${this.baseUrl}/agents/register`, {
      node_name: nodeName,
      public_ip: publicIp,
      internal_ip: internalIp,
      wg_public_key: wgPublicKey
    }).pipe(catchError(this.handleError));
  }

  // Agent heartbeat
  agentHeartbeat(nodeId: string): Observable<any> {
    return this.http.put(`${this.baseUrl}/agents/${nodeId}/heartbeat`, {})
      .pipe(catchError(this.handleError));
  }

  private handleError(error: HttpErrorResponse) {
    let errorMessage = 'An error occurred';
    
    if (error.error instanceof ErrorEvent) {
      // Client-side error
      errorMessage = error.error.message;
    } else {
      // Server-side error
      errorMessage = error.error?.error || error.statusText || 'Server error';
    }
    
    return throwError(() => new Error(errorMessage));
  }
}
