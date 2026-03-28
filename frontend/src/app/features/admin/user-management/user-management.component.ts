import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, FormGroup, Validators } from '@angular/forms';
import { MatTableModule } from '@angular/material/table';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatDialogModule, MatDialog } from '@angular/material/dialog';
import { MatIconModule } from '@angular/material/icon';
import { MatPaginatorModule } from '@angular/material/paginator';
import { MatSortModule } from '@angular/material/sort';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBarModule, MatSnackBar } from '@angular/material/snack-bar';
import { ApiService } from '../../../core/services/api.service';

interface User {
  id: string;
  username: string;
  email: string;
  role: string;
  status: 'active' | 'inactive' | 'suspended';
  createdAt: string;
}

@Component({
  selector: 'app-user-management',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    MatTableModule,
    MatButtonModule,
    MatInputModule,
    MatFormFieldModule,
    MatDialogModule,
    MatIconModule,
    MatPaginatorModule,
    MatSortModule,
    MatProgressSpinnerModule,
    MatSnackBarModule
  ],
  template: `
    <div class="user-management">
      <div class="header">
        <h2>User Management</h2>
        <button mat-raised-button color="primary" (click)="openAddUserDialog()">
          <mat-icon>add</mat-icon> Add User
        </button>
      </div>

      <mat-form-field class="search-box">
        <mat-label>Search Users</mat-label>
        <input matInput [(ngModel)]="searchTerm" (ngModelChange)="filterUsers()">
        <mat-icon matSuffix>search</mat-icon>
      </mat-form-field>

      <div *ngIf="isLoading" class="spinner">
        <mat-spinner diameter="50"></mat-spinner>
      </div>

      <table mat-table [dataSource]="filteredUsers" class="users-table" *ngIf="!isLoading">
        <ng-container matColumnDef="username">
          <th mat-header-cell *matHeaderCellDef>Username</th>
          <td mat-cell *matCellDef="let element">{{ element.username }}</td>
        </ng-container>

        <ng-container matColumnDef="email">
          <th mat-header-cell *matHeaderCellDef>Email</th>
          <td mat-cell *matCellDef="let element">{{ element.email }}</td>
        </ng-container>

        <ng-container matColumnDef="role">
          <th mat-header-cell *matHeaderCellDef>Role</th>
          <td mat-cell *matCellDef="let element">
            <span class="role-badge" [class]="element.role.toLowerCase()">
              {{ element.role }}
            </span>
          </td>
        </ng-container>

        <ng-container matColumnDef="status">
          <th mat-header-cell *matHeaderCellDef>Status</th>
          <td mat-cell *matCellDef="let element">
            <span class="status-badge" [class]="element.status">
              {{ element.status }}
            </span>
          </td>
        </ng-container>

        <ng-container matColumnDef="createdAt">
          <th mat-header-cell *matHeaderCellDef>Created</th>
          <td mat-cell *matCellDef="let element">{{ element.createdAt | date }}</td>
        </ng-container>

        <ng-container matColumnDef="actions">
          <th mat-header-cell *matHeaderCellDef>Actions</th>
          <td mat-cell *matCellDef="let element">
            <button mat-icon-button color="primary" (click)="editUser(element)" title="Edit">
              <mat-icon>edit</mat-icon>
            </button>
            <button mat-icon-button color="warn" (click)="deleteUser(element.id)" title="Delete">
              <mat-icon>delete</mat-icon>
            </button>
            <button mat-icon-button (click)="toggleUserStatus(element)" 
                    [title]="element.status === 'active' ? 'Deactivate' : 'Activate'">
              <mat-icon>{{ element.status === 'active' ? 'block' : 'check_circle' }}</mat-icon>
            </button>
          </td>
        </ng-container>

        <tr mat-header-row *matHeaderRowDef="displayedColumns"></tr>
        <tr mat-row *matRowDef="let row; columns: displayedColumns;"></tr>
      </table>

      <div class="stats" *ngIf="!isLoading">
        <div class="stat-card">
          <h4>Total Users</h4>
          <p>{{ users.length }}</p>
        </div>
        <div class="stat-card">
          <h4>Active Users</h4>
          <p>{{ users.filter(u => u.status === 'active').length }}</p>
        </div>
        <div class="stat-card">
          <h4>Admin Users</h4>
          <p>{{ users.filter(u => u.role === 'admin').length }}</p>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .user-management {
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

    .search-box {
      width: 100%;
      max-width: 400px;
      margin-bottom: 2rem;
    }

    .users-table {
      width: 100%;
      border-collapse: collapse;
      background: white;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      border-radius: 4px;
    }

    .role-badge {
      padding: 0.25rem 0.75rem;
      border-radius: 12px;
      font-size: 0.85rem;
      font-weight: 500;
    }

    .role-badge.admin {
      background: #ff9800;
      color: white;
    }

    .role-badge.user {
      background: #2196f3;
      color: white;
    }

    .role-badge.guest {
      background: #9e9e9e;
      color: white;
    }

    .status-badge {
      padding: 0.25rem 0.75rem;
      border-radius: 12px;
      font-size: 0.85rem;
      font-weight: 500;
    }

    .status-badge.active {
      background: #4caf50;
      color: white;
    }

    .status-badge.inactive {
      background: #ffc107;
      color: white;
    }

    .status-badge.suspended {
      background: #f44336;
      color: white;
    }

    .stats {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
      margin-top: 2rem;
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

    .spinner {
      display: flex;
      justify-content: center;
      padding: 3rem;
    }
  `]
})
export class UserManagementComponent implements OnInit {
  private apiService = inject(ApiService);
  private snackBar = inject(MatSnackBar);
  private dialog = inject(MatDialog);

  users: User[] = [];
  filteredUsers: User[] = [];
  searchTerm = '';
  isLoading = true;
  displayedColumns = ['username', 'email', 'role', 'status', 'createdAt', 'actions'];

  ngOnInit() {
    this.loadUsers();
  }

  loadUsers() {
    // Simulated data - replace with actual API call
    this.users = [
      {
        id: '1',
        username: 'john_admin',
        email: 'john@example.com',
        role: 'admin',
        status: 'active',
        createdAt: '2024-01-15'
      },
      {
        id: '2',
        username: 'jane_user',
        email: 'jane@example.com',
        role: 'user',
        status: 'active',
        createdAt: '2024-01-20'
      },
      {
        id: '3',
        username: 'bob_user',
        email: 'bob@example.com',
        role: 'user',
        status: 'inactive',
        createdAt: '2024-02-01'
      }
    ];
    this.filteredUsers = this.users;
    this.isLoading = false;
  }

  filterUsers() {
    const term = this.searchTerm.toLowerCase();
    this.filteredUsers = this.users.filter(user =>
      user.username.toLowerCase().includes(term) ||
      user.email.toLowerCase().includes(term)
    );
  }

  openAddUserDialog() {
    this.snackBar.open('Add user dialog would open here', 'Close', { duration: 3000 });
  }

  editUser(user: User) {
    this.snackBar.open(`Editing: ${user.username}`, 'Close', { duration: 3000 });
  }

  deleteUser(userId: string) {
    if (confirm('Are you sure you want to delete this user?')) {
      this.users = this.users.filter(u => u.id !== userId);
      this.filterUsers();
      this.snackBar.open('User deleted successfully', 'Close', { duration: 3000 });
    }
  }

  toggleUserStatus(user: User) {
    user.status = user.status === 'active' ? 'inactive' : 'active';
    this.snackBar.open(`User ${user.status}`, 'Close', { duration: 3000 });
  }
}
