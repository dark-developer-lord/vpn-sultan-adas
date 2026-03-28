import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { ApiService } from '../../../core/services/api.service';
import { MatTableModule } from '@angular/material/table';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatCardModule } from '@angular/material/card';
import { MatTooltipModule } from '@angular/material/tooltip';

interface Node {
  id: string;
  name: string;
  region: string;
  public_key: string;
  status: string;
  last_heartbeat: string;
}

@Component({
  selector: 'app-nodes-list',
  standalone: true,
  imports: [
    CommonModule,
    MatTableModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatCardModule,
    MatTooltipModule,
  ],
  template: `
    <mat-card class="container">
      <mat-card-header>
        <mat-card-title>VPN Nodes</mat-card-title>
        <mat-card-subtitle>Available VPN endpoints for your network</mat-card-subtitle>
      </mat-card-header>

      <mat-card-content>
        <div *ngIf="isLoading" class="loading">
          <mat-spinner diameter="50"></mat-spinner>
          <p>Loading nodes...</p>
        </div>

        <div *ngIf="!isLoading && nodes.length === 0" class="no-data">
          <p>No nodes available</p>
        </div>

        <table mat-table [dataSource]="nodes" *ngIf="!isLoading && nodes.length > 0" class="nodes-table">
          <!-- Name Column -->
          <ng-container matColumnDef="name">
            <th mat-header-cell *matHeaderCellDef>Name</th>
            <td mat-cell *matCellDef="let node">{{ node.name }}</td>
          </ng-container>

          <!-- Region Column -->
          <ng-container matColumnDef="region">
            <th mat-header-cell *matHeaderCellDef>Region</th>
            <td mat-cell *matCellDef="let node">{{ node.region }}</td>
          </ng-container>

          <!-- Status Column -->
          <ng-container matColumnDef="status">
            <th mat-header-cell *matHeaderCellDef>Status</th>
            <td mat-cell *matCellDef="let node">
              <span class="status-badge" [class.online]="node.status === 'online'" [class.offline]="node.status === 'offline'">
                {{ node.status | uppercase }}
              </span>
            </td>
          </ng-container>

          <!-- Last Heartbeat Column -->
          <ng-container matColumnDef="lastHeartbeat">
            <th mat-header-cell *matHeaderCellDef>Last Heartbeat</th>
            <td mat-cell *matCellDef="let node">{{ formatDate(node.last_heartbeat) }}</td>
          </ng-container>

          <!-- Actions Column -->
          <ng-container matColumnDef="actions">
            <th mat-header-cell *matHeaderCellDef>Actions</th>
            <td mat-cell *matCellDef="let node">
              <button mat-icon-button (click)="copyPublicKey(node)" matTooltip="Copy public key">
                <mat-icon>content_copy</mat-icon>
              </button>
            </td>
          </ng-container>

          <tr mat-header-row *matHeaderRowDef="displayedColumns"></tr>
          <tr mat-row *matRowDef="let row; columns: displayedColumns;"></tr>
        </table>
      </mat-card-content>
    </mat-card>
  `,
  styles: [`
    .container {
      max-width: 1000px;
      margin: 2rem auto;
    }

    mat-card-header {
      margin-bottom: 1.5rem;
    }

    mat-card-title {
      font-size: 1.5rem;
      font-weight: 500;
      margin: 0;
    }

    mat-card-subtitle {
      color: #999;
      font-size: 0.9rem;
      margin: 0.5rem 0 0 0;
    }

    .loading, .no-data {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      padding: 3rem;
      color: #666;
    }

    .nodes-table {
      width: 100%;
      margin-top: 1rem;
    }

    th {
      background-color: #f5f5f5;
      font-weight: 600;
      color: #333;
    }

    td {
      padding: 0.75rem 1rem;
    }

    .status-badge {
      display: inline-block;
      padding: 0.25rem 0.75rem;
      border-radius: 20px;
      font-size: 0.75rem;
      font-weight: 600;
      background-color: #e0e0e0;
      color: #666;
    }

    .status-badge.online {
      background-color: #c8e6c9;
      color: #2e7d32;
    }

    .status-badge.offline {
      background-color: #ffcdd2;
      color: #c62828;
    }

    button {
      color: #2196F3;
    }

    button:hover {
      color: #1976D2;
    }
  `]
})
export class NodesListComponent implements OnInit {
  private api = inject(ApiService);

  isLoading = true;
  nodes: Node[] = [];
  displayedColumns = ['name', 'region', 'status', 'lastHeartbeat', 'actions'];

  ngOnInit(): void {
    this.loadNodes();
  }

  loadNodes(): void {
    this.isLoading = true;
    this.api.getNodes().subscribe({
      next: (response) => {
        this.nodes = response.data || [];
        this.isLoading = false;
      },
      error: (error) => {
        console.error('Failed to load nodes:', error);
        this.isLoading = false;
      },
    });
  }

  formatDate(dateString: string): string {
    if (!dateString) return 'Never';
    try {
      const date = new Date(dateString);
      return date.toLocaleString();
    } catch {
      return dateString;
    }
  }

  copyPublicKey(node: Node): void {
    navigator.clipboard.writeText(node.public_key).then(() => {
      alert('Public key copied to clipboard!');
    }).catch(() => {
      alert('Failed to copy public key');
    });
  }
}
