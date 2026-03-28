import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { ApiService } from '../../../core/services/api.service';
import { MatButtonModule } from '@angular/material/button';
import { MatTableModule } from '@angular/material/table';
import { MatInputModule } from '@angular/material/input';
import { MatSelectModule } from '@angular/material/select';
import { MatCardModule } from '@angular/material/card';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';

@Component({
  selector: 'app-peers-list',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    MatButtonModule,
    MatTableModule,
    MatInputModule,
    MatSelectModule,
    MatCardModule,
    MatIconModule,
    MatProgressSpinnerModule,
  ],
  template: `
    <div class="peers-container">
      <div class="header">
        <h1>VPN Peers</h1>
        <button mat-raised-button color="primary" (click)="openCreateForm()">
          <mat-icon>add</mat-icon>
          Create New Peer
        </button>
      </div>

      <mat-card *ngIf="showCreateForm" class="create-form">
        <mat-card-header>
          <mat-card-title>Create New Peer</mat-card-title>
        </mat-card-header>
        <mat-card-content>
          <div class="form-group">
            <mat-form-field>
              <mat-label>Peer Name</mat-label>
              <input matInput [(ngModel)]="newPeer.name" placeholder="e.g., laptop-1, phone" />
            </mat-form-field>
          </div>
          <div class="form-group">
            <mat-form-field>
              <mat-label>Select Node</mat-label>
              <mat-select [(ngModel)]="newPeer.nodeId">
                <mat-option *ngFor="let node of availableNodes" [value]="node.id">
                  {{ node.name }} ({{ node.public_ip }})
                </mat-option>
              </mat-select>
            </mat-form-field>
          </div>
          <div class="form-actions">
            <button mat-raised-button color="primary" (click)="createPeer()" [disabled]="isLoading">
              Create
            </button>
            <button mat-button (click)="closeCreateForm()">Cancel</button>
          </div>
        </mat-card-content>
      </mat-card>

      <mat-spinner *ngIf="isLoading" diameter="50"></mat-spinner>

      <div *ngIf="!isLoading && peers.length === 0" class="no-peers">
        <p>No peers yet. Create one to get started!</p>
      </div>

      <mat-table [dataSource]="peers" class="peers-table" *ngIf="!isLoading && peers.length > 0">
        <!-- Name Column -->
        <ng-container matColumnDef="name">
          <mat-header-cell *matHeaderCellDef>Name</mat-header-cell>
          <mat-cell *matCellDef="let peer">{{ peer.name }}</mat-cell>
        </ng-container>

        <!-- Status Column -->
        <ng-container matColumnDef="status">
          <mat-header-cell *matHeaderCellDef>Status</mat-header-cell>
          <mat-cell *matCellDef="let peer">
            <span [ngClass]="'status-' + peer.status">{{ peer.status }}</span>
          </mat-cell>
        </ng-container>

        <!-- Created Column -->
        <ng-container matColumnDef="created">
          <mat-header-cell *matHeaderCellDef>Created</mat-header-cell>
          <mat-cell *matCellDef="let peer">{{ formatDate(peer.created_at) }}</mat-cell>
        </ng-container>

        <!-- Actions Column -->
        <ng-container matColumnDef="actions">
          <mat-header-cell *matHeaderCellDef>Actions</mat-header-cell>
          <mat-cell *matCellDef="let peer">
            <button mat-icon-button (click)="downloadConfig(peer.id)" title="Download Config">
              <mat-icon>cloud_download</mat-icon>
            </button>
            <button mat-icon-button (click)="deletePeer(peer.id)" title="Delete Peer" color="warn">
              <mat-icon>delete</mat-icon>
            </button>
          </mat-cell>
        </ng-container>

        <mat-header-row *matHeaderRowDef="displayedColumns"></mat-header-row>
        <mat-row *matRowDef="let row; columns: displayedColumns;"></mat-row>
      </mat-table>
    </div>
  `,
  styles: [`
    .peers-container {
      max-width: 1000px;
      margin: 0 auto;
      padding: 2rem;
    }
    .header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 2rem;
    }
    .create-form {
      margin-bottom: 2rem;
    }
    .form-group {
      margin-bottom: 1rem;
    }
    .form-actions {
      display: flex;
      gap: 1rem;
      margin-top: 1.5rem;
    }
    .peers-table {
      width: 100%;
      margin-top: 1rem;
    }
    .no-peers {
      text-align: center;
      padding: 3rem;
      color: #999;
    }
    .status-active {
      color: green;
      font-weight: bold;
    }
    .status-revoked {
      color: red;
    }
    mat-form-field {
      width: 100%;
      max-width: 300px;
    }
  `]
})
export class PeersListComponent implements OnInit {
  private api = inject(ApiService);

  peers: any[] = [];
  availableNodes: any[] = [];
  isLoading = false;
  showCreateForm = false;
  
  newPeer = {
    name: '',
    nodeId: '',
  };

  displayedColumns = ['name', 'status', 'created', 'actions'];

  ngOnInit(): void {
    this.loadPeers();
    this.loadNodes();
  }

  loadPeers(): void {
    this.isLoading = true;
    this.api.getPeers().subscribe({
      next: (response) => {
        this.peers = response.data || [];
        this.isLoading = false;
      },
      error: (error) => {
        console.error('Failed to load peers:', error);
        this.isLoading = false;
      },
    });
  }

  loadNodes(): void {
    this.api.getNodes().subscribe({
      next: (response) => {
        this.availableNodes = response.data || [];
      },
      error: (error) => {
        console.error('Failed to load nodes:', error);
      },
    });
  }

  openCreateForm(): void {
    this.showCreateForm = true;
  }

  closeCreateForm(): void {
    this.showCreateForm = false;
    this.newPeer = { name: '', nodeId: '' };
  }

  createPeer(): void {
    if (!this.newPeer.name || !this.newPeer.nodeId) {
      alert('Please fill in all fields');
      return;
    }

    this.isLoading = true;
    this.api.createPeer(this.newPeer.nodeId, this.newPeer.name).subscribe({
      next: (response) => {
        this.peers.push(response.data);
        this.closeCreateForm();
        this.isLoading = false;
        alert('Peer created successfully!');
      },
      error: (error) => {
        console.error('Failed to create peer:', error);
        alert('Failed to create peer: ' + error.message);
        this.isLoading = false;
      },
    });
  }

  downloadConfig(peerId: string): void {
    this.api.getPeerConfig(peerId).subscribe({
      next: (response) => {
        const config = response.data.config;
        const element = document.createElement('a');
        element.setAttribute('href', 'data:text/plain;charset=utf-8,' + encodeURIComponent(config));
        element.setAttribute('download', `peer-${peerId}.conf`);
        element.style.display = 'none';
        document.body.appendChild(element);
        element.click();
        document.body.removeChild(element);
      },
      error: (error) => {
        console.error('Failed to download config:', error);
        alert('Failed to download config: ' + error.message);
      },
    });
  }

  deletePeer(peerId: string): void {
    if (!confirm('Are you sure you want to delete this peer?')) {
      return;
    }

    this.api.revokePeer(peerId).subscribe({
      next: () => {
        this.peers = this.peers.filter(p => p.id !== peerId);
        alert('Peer deleted successfully!');
      },
      error: (error) => {
        console.error('Failed to delete peer:', error);
        alert('Failed to delete peer: ' + error.message);
      },
    });
  }

  formatDate(dateString: string): string {
    return new Date(dateString).toLocaleDateString();
  }
}
