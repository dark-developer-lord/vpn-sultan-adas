import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ApiService } from '../../core/services/api.service';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

@Component({
  selector: 'app-dashboard',
  standalone: true,
  imports: [CommonModule, MatProgressSpinnerModule],
  template: `
    <div class="dashboard">
      <h1>Dashboard</h1>
      
      <mat-spinner *ngIf="isLoading" diameter="50"></mat-spinner>

      <div class="stats-grid" *ngIf="!isLoading">
        <div class="stat-card">
          <h3>Active Peers</h3>
          <p class="big-number">{{ stats.activePeers }}</p>
        </div>
        <div class="stat-card">
          <h3>Online Nodes</h3>
          <p class="big-number">{{ stats.onlineNodes }}</p>
        </div>
        <div class="stat-card">
          <h3>Total Data</h3>
          <p class="big-number">{{ stats.totalData }} GB</p>
        </div>
        <div class="stat-card">
          <h3>Subscription Plan</h3>
          <p class="big-number capitalize">{{ stats.subscriptionPlan }}</p>
        </div>
      </div>

      <div class="quick-actions" *ngIf="!isLoading">
        <h2>Quick Stats</h2>
        <ul>
          <li>Max Peers: <strong>{{ stats.maxPeers }}</strong></li>
          <li>Current Peers: <strong>{{ stats.currentPeers }}</strong> / {{ stats.maxPeers }}</li>
          <li>Last Updated: <strong>{{ stats.lastUpdated }}</strong></li>
        </ul>
      </div>
    </div>
  `,
  styles: [`
    .dashboard {
      max-width: 1200px;
      margin: 0 auto;
      padding: 2rem;
    }

    .stats-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: 2rem;
      margin-top: 2rem;
    }

    .stat-card {
      background: white;
      padding: 1.5rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      border-left: 4px solid #2196F3;
    }

    .stat-card h3 {
      color: #666;
      margin: 0 0 1rem 0;
      font-size: 0.9rem;
      text-transform: uppercase;
      letter-spacing: 0.5px;
    }

    .big-number {
      font-size: 2.5rem;
      font-weight: bold;
      color: #333;
      margin: 0;
    }

    .quick-actions {
      margin-top: 3rem;
      background: white;
      padding: 2rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .quick-actions h2 {
      margin-top: 0;
      color: #333;
    }

    .quick-actions ul {
      list-style: none;
      padding: 0;
      margin: 0;
    }

    .quick-actions li {
      padding: 0.75rem 0;
      border-bottom: 1px solid #eee;
      display: flex;
      justify-content: space-between;
    }

    .quick-actions li:last-child {
      border-bottom: none;
    }

    .capitalize {
      text-transform: capitalize;
    }
  `]
})
export class DashboardComponent implements OnInit {
  private api = inject(ApiService);

  isLoading = true;
  stats = {
    activePeers: 0,
    onlineNodes: 0,
    totalData: 0,
    subscriptionPlan: 'free',
    maxPeers: 2,
    currentPeers: 0,
    lastUpdated: 'Never',
  };

  ngOnInit(): void {
    this.loadStats();
  }

  loadStats(): void {
    this.isLoading = true;

    // Load peers
    this.api.getPeers().subscribe({
      next: (response) => {
        const peers = response.data || [];
        this.stats.activePeers = peers.filter((p: any) => p.status === 'active').length;
        this.stats.currentPeers = peers.length;
      },
      error: (error) => {
        console.error('Failed to load peers:', error);
      },
    });

    // Load nodes
    this.api.getNodes().subscribe({
      next: (response) => {
        const nodes = response.data || [];
        this.stats.onlineNodes = nodes.filter((n: any) => n.status === 'online').length;
      },
      error: (error) => {
        console.error('Failed to load nodes:', error);
      },
      complete: () => {
        this.isLoading = false;
        this.stats.lastUpdated = new Date().toLocaleTimeString();
      },
    });
  }
}
