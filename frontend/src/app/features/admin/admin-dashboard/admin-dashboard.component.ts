import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { MatTabsModule } from '@angular/material/tabs';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { MatCardModule } from '@angular/material/card';
import { UserManagementComponent } from '../user-management/user-management.component';
import { ServerManagementComponent } from '../server-management/server-management.component';
import { PolicyManagementComponent } from '../policy-management/policy-management.component';
import { AdminAnalyticsComponent } from '../analytics/admin-analytics.component';
import { AdminSettingsComponent } from '../settings/admin-settings.component';

@Component({
  selector: 'app-admin-dashboard',
  standalone: true,
  imports: [
    CommonModule,
    MatTabsModule,
    MatIconModule,
    MatButtonModule,
    MatCardModule,
    UserManagementComponent,
    ServerManagementComponent,
    PolicyManagementComponent,
    AdminAnalyticsComponent,
    AdminSettingsComponent
  ],
  template: `
    <div class="admin-dashboard-container">
      <div class="admin-header">
        <h1>
          <mat-icon>admin_panel_settings</mat-icon>
          Administration Panel
        </h1>
        <p class="subtitle">Manage users, servers, policies, analytics, and system settings</p>
      </div>

      <mat-tab-group>
        <!-- Users Tab -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>people</mat-icon>
            <span>Users</span>
          </ng-template>
          <app-user-management></app-user-management>
        </mat-tab>

        <!-- Servers Tab -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>storage</mat-icon>
            <span>Servers</span>
          </ng-template>
          <app-server-management></app-server-management>
        </mat-tab>

        <!-- Policies Tab -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>policy</mat-icon>
            <span>Policies</span>
          </ng-template>
          <app-policy-management></app-policy-management>
        </mat-tab>

        <!-- Analytics Tab -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>analytics</mat-icon>
            <span>Analytics</span>
          </ng-template>
          <app-admin-analytics></app-admin-analytics>
        </mat-tab>

        <!-- Settings Tab -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>settings</mat-icon>
            <span>Settings</span>
          </ng-template>
          <app-admin-settings></app-admin-settings>
        </mat-tab>
      </mat-tab-group>
    </div>
  `,
  styles: [`
    .admin-dashboard-container {
      padding: 0;
      background: #f5f5f5;
      min-height: 100vh;
    }

    .admin-header {
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: white;
      padding: 2rem;
      margin-bottom: 2rem;
      box-shadow: 0 4px 6px rgba(0,0,0,0.1);
    }

    .admin-header h1 {
      margin: 0;
      display: flex;
      align-items: center;
      gap: 1rem;
      font-size: 2rem;
    }

    .admin-header mat-icon {
      font-size: 2.5rem;
      width: 2.5rem;
      height: 2.5rem;
    }

    .subtitle {
      margin: 0.5rem 0 0 0;
      opacity: 0.9;
      font-size: 0.95rem;
    }

    mat-tab-group {
      background: white;
      margin: 0 1rem;
      border-radius: 4px;
      box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    }
  `]
})
export class AdminDashboardComponent implements OnInit {
  ngOnInit() {
    // Initialize admin dashboard
  }
}
