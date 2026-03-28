import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatProgressBarModule } from '@angular/material/progress-bar';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { ApiService } from '../../../core/services/api.service';

interface Server {
  id: string;
  name: string;
  ipAddress: string;
  region: string;
  status: 'online' | 'offline' | 'maintenance';
  cpuUsage: number;
  memoryUsage: number;
  bandwidthUsage: number;
  activeConnections: number;
  lastHealthCheck: string;
}

@Component({
  selector: 'app-server-management',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatFormFieldModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatSlideToggleModule,
    MatProgressBarModule,
    MatSnackBarModule
  ],
  template: `
    <div class="server-management">
      <div class="header">
        <h2>Server Management</h2>
        <button mat-raised-button color="primary">
          <mat-icon>add</mat-icon> Add Server
        </button>
      </div>

      <div *ngIf="isLoading" class="spinner">
        <mat-spinner diameter="50"></mat-spinner>
      </div>

      <div class="servers-grid" *ngIf="!isLoading">
        <mat-card class="server-card" *ngFor="let server of servers">
          <mat-card-header>
            <div class="server-title">
              <h3>{{ server.name }}</h3>
              <span class="status-badge" [class]="server.status">
                <mat-icon>circle</mat-icon> {{ server.status }}
              </span>
            </div>
          </mat-card-header>

          <mat-card-content>
            <div class="server-info">
              <div class="info-row">
                <span class="label">IP Address:</span>
                <span class="value">{{ server.ipAddress }}</span>
              </div>
              <div class="info-row">
                <span class="label">Region:</span>
                <span class="value">{{ server.region }}</span>
              </div>
              <div class="info-row">
                <span class="label">Active Connections:</span>
                <span class="value">{{ server.activeConnections }}</span>
              </div>
              <div class="info-row">
                <span class="label">Last Health Check:</span>
                <span class="value">{{ server.lastHealthCheck }}</span>
              </div>
            </div>

            <div class="metrics">
              <div class="metric">
                <label>CPU Usage</label>
                <mat-progress-bar mode="determinate" 
                                   [value]="server.cpuUsage"
                                   [color]="server.cpuUsage > 80 ? 'warn' : 'accent'">
                </mat-progress-bar>
                <span class="percentage">{{ server.cpuUsage }}%</span>
              </div>

              <div class="metric">
                <label>Memory Usage</label>
                <mat-progress-bar mode="determinate" 
                                   [value]="server.memoryUsage"
                                   [color]="server.memoryUsage > 80 ? 'warn' : 'accent'">
                </mat-progress-bar>
                <span class="percentage">{{ server.memoryUsage }}%</span>
              </div>

              <div class="metric">
                <label>Bandwidth Usage</label>
                <mat-progress-bar mode="determinate" 
                                   [value]="server.bandwidthUsage"
                                   [color]="server.bandwidthUsage > 80 ? 'warn' : 'accent'">
                </mat-progress-bar>
                <span class="percentage">{{ server.bandwidthUsage }}%</span>
              </div>
            </div>
          </mat-card-content>

          <mat-card-actions>
            <button mat-button (click)="viewDetails(server)">
              <mat-icon>visibility</mat-icon> View Details
            </button>
            <button mat-button (click)="restartServer(server)">
              <mat-icon>restart_alt</mat-icon> Restart
            </button>
            <button mat-button color="warn" (click)="removeServer(server.id)">
              <mat-icon>delete</mat-icon> Remove
            </button>
          </mat-card-actions>
        </mat-card>
      </div>

      <div class="stats" *ngIf="!isLoading">
        <div class="stat-card">
          <h4>Total Servers</h4>
          <p>{{ servers.length }}</p>
        </div>
        <div class="stat-card">
          <h4>Online</h4>
          <p class="success">{{ servers.filter(s => s.status === 'online').length }}</p>
        </div>
        <div class="stat-card">
          <h4>Offline</h4>
          <p class="error">{{ servers.filter(s => s.status === 'offline').length }}</p>
        </div>
        <div class="stat-card">
          <h4>Avg CPU Usage</h4>
          <p>{{ (servers.reduce((a, s) => a + s.cpuUsage, 0) / servers.length).toFixed(1) }}%</p>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .server-management {
      padding: 2rem;
    }

    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 2rem;
    }

    .header h2 {
      margin: 0;
      color: #333;
    }

    .servers-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
      gap: 2rem;
      margin-bottom: 2rem;
    }

    .server-card {
      transition: box-shadow 0.3s ease;
    }

    .server-card:hover {
      box-shadow: 0 8px 16px rgba(0,0,0,0.15);
    }

    .server-title {
      display: flex;
      justify-content: space-between;
      align-items: center;
      gap: 1rem;
    }

    .server-title h3 {
      margin: 0;
      color: #333;
    }

    .status-badge {
      display: flex;
      align-items: center;
      gap: 0.25rem;
      padding: 0.25rem 0.75rem;
      border-radius: 12px;
      font-size: 0.8rem;
      font-weight: 500;
    }

    .status-badge.online {
      background: #4caf50;
      color: white;
    }

    .status-badge.offline {
      background: #f44336;
      color: white;
    }

    .status-badge.maintenance {
      background: #ffc107;
      color: white;
    }

    .server-info {
      margin-bottom: 1.5rem;
    }

    .info-row {
      display: flex;
      justify-content: space-between;
      padding: 0.5rem 0;
      font-size: 0.9rem;
    }

    .info-row .label {
      color: #666;
      font-weight: 500;
    }

    .info-row .value {
      color: #333;
      font-family: monospace;
    }

    .metrics {
      gap: 1rem;
    }

    .metric {
      margin-bottom: 1rem;
    }

    .metric label {
      display: block;
      font-size: 0.85rem;
      font-weight: 500;
      color: #666;
      margin-bottom: 0.5rem;
    }

    .percentage {
      font-size: 0.8rem;
      color: #999;
      float: right;
    }

    .stats {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
    }

    .stat-card {
      background: white;
      padding: 1.5rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      text-align: center;
    }

    .stat-card h4 {
      margin: 0 0 0.5rem 0;
      color: #666;
      font-size: 0.9rem;
      text-transform: uppercase;
    }

    .stat-card p {
      margin: 0;
      font-size: 2rem;
      font-weight: bold;
      color: #333;
    }

    .stat-card p.success {
      color: #4caf50;
    }

    .stat-card p.error {
      color: #f44336;
    }

    .spinner {
      display: flex;
      justify-content: center;
      padding: 3rem;
    }
  `]
})
export class ServerManagementComponent implements OnInit {
  private apiService = inject(ApiService);
  private snackBar = inject(MatSnackBar);

  servers: Server[] = [];
  isLoading = true;

  ngOnInit() {
    this.loadServers();
  }

  loadServers() {
    // Simulated data - replace with actual API call
    this.servers = [
      {
        id: '1',
        name: 'US-East-1',
        ipAddress: '192.168.1.10',
        region: 'United States (N. Virginia)',
        status: 'online',
        cpuUsage: 45,
        memoryUsage: 62,
        bandwidthUsage: 38,
        activeConnections: 1250,
        lastHealthCheck: '2 minutes ago'
      },
      {
        id: '2',
        name: 'EU-West-1',
        ipAddress: '192.168.1.20',
        region: 'Ireland',
        status: 'online',
        cpuUsage: 72,
        memoryUsage: 58,
        bandwidthUsage: 65,
        activeConnections: 980,
        lastHealthCheck: '1 minute ago'
      },
      {
        id: '3',
        name: 'AP-South-1',
        ipAddress: '192.168.1.30',
        region: 'Asia Pacific (Singapore)',
        status: 'maintenance',
        cpuUsage: 15,
        memoryUsage: 25,
        bandwidthUsage: 10,
        activeConnections: 0,
        lastHealthCheck: '5 minutes ago'
      }
    ];
    this.isLoading = false;
  }

  viewDetails(server: Server) {
    this.snackBar.open(`Viewing details for ${server.name}`, 'Close', { duration: 3000 });
  }

  restartServer(server: Server) {
    if (confirm(`Restart ${server.name}? This will temporarily interrupt active connections.`)) {
      this.snackBar.open(`Restarting ${server.name}...`, 'Close', { duration: 3000 });
    }
  }

  removeServer(serverId: string) {
    if (confirm('Are you sure you want to remove this server?')) {
      this.servers = this.servers.filter(s => s.id !== serverId);
      this.snackBar.open('Server removed successfully', 'Close', { duration: 3000 });
    }
  }
}
