import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, FormGroup, Validators } from '@angular/forms';
import { MatTableModule } from '@angular/material/table';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSelectModule } from '@angular/material/select';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatChipsModule } from '@angular/material/chips';
import { ApiService } from '../../../core/services/api.service';

interface Policy {
  id: string;
  name: string;
  description: string;
  rules: PolicyRule[];
  assignedTo: string[];
  enabled: boolean;
  priority: number;
  createdAt: string;
}

interface PolicyRule {
  id: string;
  type: 'allow' | 'deny' | 'rate-limit';
  condition: string;
  action: string;
}

@Component({
  selector: 'app-policy-management',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    MatTableModule,
    MatButtonModule,
    MatInputModule,
    MatFormFieldModule,
    MatSelectModule,
    MatSlideToggleModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatSnackBarModule,
    MatChipsModule
  ],
  template: `
    <div class="policy-management">
      <div class="header">
        <h2>Policy Management</h2>
        <button mat-raised-button color="primary" (click)="openCreatePolicyDialog()">
          <mat-icon>add</mat-icon> Create Policy
        </button>
      </div>

      <div *ngIf="isLoading" class="spinner">
        <mat-spinner diameter="50"></mat-spinner>
      </div>

      <table mat-table [dataSource]="policies" class="policies-table" *ngIf="!isLoading">
        <ng-container matColumnDef="name">
          <th mat-header-cell *matHeaderCellDef>Policy Name</th>
          <td mat-cell *matCellDef="let element">
            <div class="policy-name">
              <strong>{{ element.name }}</strong>
              <small>{{ element.description }}</small>
            </div>
          </td>
        </ng-container>

        <ng-container matColumnDef="priority">
          <th mat-header-cell *matHeaderCellDef>Priority</th>
          <td mat-cell *matCellDef="let element">
            <span class="priority-badge" [class]="'priority-' + element.priority">
              {{ element.priority }}
            </span>
          </td>
        </ng-container>

        <ng-container matColumnDef="rules">
          <th mat-header-cell *matHeaderCellDef>Rules</th>
          <td mat-cell *matCellDef="let element">
            <mat-chip-set>
              <mat-chip *ngFor="let rule of element.rules" 
                        [class]="'rule-type-' + rule.type">
                {{ rule.type }}
              </mat-chip>
            </mat-chip-set>
          </td>
        </ng-container>

        <ng-container matColumnDef="assignedTo">
          <th mat-header-cell *matHeaderCellDef>Assigned To</th>
          <td mat-cell *matCellDef="let element">
            <span class="assigned-count">{{ element.assignedTo.length }} user(s)</span>
          </td>
        </ng-container>

        <ng-container matColumnDef="enabled">
          <th mat-header-cell *matHeaderCellDef>Enabled</th>
          <td mat-cell *matCellDef="let element">
            <mat-slide-toggle [checked]="element.enabled" 
                              (change)="togglePolicy(element)">
            </mat-slide-toggle>
          </td>
        </ng-container>

        <ng-container matColumnDef="actions">
          <th mat-header-cell *matHeaderCellDef>Actions</th>
          <td mat-cell *matCellDef="let element">
            <button mat-icon-button color="primary" (click)="editPolicy(element)" title="Edit">
              <mat-icon>edit</mat-icon>
            </button>
            <button mat-icon-button color="accent" (click)="clonePolicy(element)" title="Clone">
              <mat-icon>content_copy</mat-icon>
            </button>
            <button mat-icon-button color="warn" (click)="deletePolicy(element.id)" title="Delete">
              <mat-icon>delete</mat-icon>
            </button>
          </td>
        </ng-container>

        <tr mat-header-row *matHeaderRowDef="displayedColumns"></tr>
        <tr mat-row *matRowDef="let row; columns: displayedColumns;"></tr>
      </table>

      <div class="stats" *ngIf="!isLoading">
        <div class="stat-card">
          <h4>Total Policies</h4>
          <p>{{ policies.length }}</p>
        </div>
        <div class="stat-card">
          <h4>Enabled</h4>
          <p>{{ policies.filter(p => p.enabled).length }}</p>
        </div>
        <div class="stat-card">
          <h4>Total Rules</h4>
          <p>{{ policies.reduce((sum, p) => sum + p.rules.length, 0) }}</p>
        </div>
        <div class="stat-card">
          <h4>Average Priority</h4>
          <p>{{ (policies.reduce((sum, p) => sum + p.priority, 0) / policies.length).toFixed(1) }}</p>
        </div>
      </div>

      <div class="policy-rules-preview" *ngIf="!isLoading">
        <h3>Quick Policy Rules Summary</h3>
        <div class="rules-list">
          <div class="rule-item" *ngFor="let policy of policies; let i = index">
            <strong>{{ policy.name }}</strong>
            <span class="status" [class.enabled]="policy.enabled">{{ policy.enabled ? 'Enabled' : 'Disabled' }}</span>
            <ul>
              <li *ngFor="let rule of policy.rules">
                {{ rule.type | uppercase }}: {{ rule.condition }} → {{ rule.action }}
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .policy-management {
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

    .policies-table {
      width: 100%;
      border-collapse: collapse;
      background: white;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      border-radius: 4px;
      margin-bottom: 2rem;
    }

    .policy-name {
      display: flex;
      flex-direction: column;
    }

    .policy-name strong {
      color: #333;
    }

    .policy-name small {
      color: #999;
      margin-top: 0.25rem;
    }

    .priority-badge {
      padding: 0.25rem 0.75rem;
      border-radius: 12px;
      font-size: 0.85rem;
      font-weight: 500;
    }

    .priority-badge.priority-1 {
      background: #4caf50;
      color: white;
    }

    .priority-badge.priority-2 {
      background: #2196f3;
      color: white;
    }

    .priority-badge.priority-3 {
      background: #ff9800;
      color: white;
    }

    .rule-type-allow {
      background: #4caf50;
      color: white;
    }

    .rule-type-deny {
      background: #f44336;
      color: white;
    }

    .rule-type-rate-limit {
      background: #ffc107;
      color: white;
    }

    .assigned-count {
      padding: 0.25rem 0.75rem;
      background: #e0e0e0;
      border-radius: 12px;
      font-size: 0.85rem;
    }

    .stats {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
      gap: 1rem;
      margin-bottom: 2rem;
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

    .policy-rules-preview {
      background: white;
      padding: 2rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .policy-rules-preview h3 {
      margin-top: 0;
      color: #333;
    }

    .rules-list {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
      gap: 1.5rem;
    }

    .rule-item {
      display: flex;
      flex-direction: column;
      gap: 0.75rem;
      padding: 1rem;
      background: #f5f5f5;
      border-radius: 4px;
      border-left: 4px solid #2196f3;
    }

    .rule-item strong {
      color: #333;
    }

    .status {
      font-size: 0.85rem;
      padding: 0.25rem 0.5rem;
      border-radius: 4px;
      background: #e0e0e0;
      width: fit-content;
    }

    .status.enabled {
      background: #c8e6c9;
      color: #2e7d32;
    }

    .rule-item ul {
      margin: 0.5rem 0 0 0;
      padding-left: 1.5rem;
      font-size: 0.85rem;
      color: #666;
    }

    .rule-item li {
      margin: 0.25rem 0;
    }

    .spinner {
      display: flex;
      justify-content: center;
      padding: 3rem;
    }
  `]
})
export class PolicyManagementComponent implements OnInit {
  private apiService = inject(ApiService);
  private snackBar = inject(MatSnackBar);

  policies: Policy[] = [];
  isLoading = true;
  displayedColumns = ['name', 'priority', 'rules', 'assignedTo', 'enabled', 'actions'];

  ngOnInit() {
    this.loadPolicies();
  }

  loadPolicies() {
    // Simulated data - replace with actual API call
    this.policies = [
      {
        id: '1',
        name: 'Basic User Policy',
        description: 'Default policy for regular users',
        rules: [
          { id: '1', type: 'allow', condition: 'role=user', action: 'allow_basic_access' },
          { id: '2', type: 'rate-limit', condition: 'requests/min > 100', action: 'throttle' }
        ],
        assignedTo: ['user1', 'user2', 'user3'],
        enabled: true,
        priority: 2,
        createdAt: '2024-01-10'
      },
      {
        id: '2',
        name: 'Admin Policy',
        description: 'Full access policy for administrators',
        rules: [
          { id: '3', type: 'allow', condition: 'role=admin', action: 'allow_all' }
        ],
        assignedTo: ['admin1'],
        enabled: true,
        priority: 1,
        createdAt: '2024-01-15'
      },
      {
        id: '3',
        name: 'Guest Policy',
        description: 'Limited access for guest users',
        rules: [
          { id: '4', type: 'deny', condition: 'role=guest', action: 'deny_admin_access' },
          { id: '5', type: 'rate-limit', condition: 'requests/min > 50', action: 'block' }
        ],
        assignedTo: ['guest1', 'guest2'],
        enabled: false,
        priority: 3,
        createdAt: '2024-01-20'
      }
    ];
    this.isLoading = false;
  }

  openCreatePolicyDialog() {
    this.snackBar.open('Create policy dialog would open here', 'Close', { duration: 3000 });
  }

  editPolicy(policy: Policy) {
    this.snackBar.open(`Editing policy: ${policy.name}`, 'Close', { duration: 3000 });
  }

  clonePolicy(policy: Policy) {
    const newPolicy = {
      ...policy,
      id: Math.random().toString(),
      name: `${policy.name} (Copy)`
    };
    this.policies.push(newPolicy);
    this.snackBar.open(`Policy cloned: ${newPolicy.name}`, 'Close', { duration: 3000 });
  }

  togglePolicy(policy: Policy) {
    policy.enabled = !policy.enabled;
    this.snackBar.open(
      `Policy ${policy.enabled ? 'enabled' : 'disabled'}: ${policy.name}`,
      'Close',
      { duration: 3000 }
    );
  }

  deletePolicy(policyId: string) {
    if (confirm('Are you sure you want to delete this policy?')) {
      this.policies = this.policies.filter(p => p.id !== policyId);
      this.snackBar.open('Policy deleted successfully', 'Close', { duration: 3000 });
    }
  }
}
