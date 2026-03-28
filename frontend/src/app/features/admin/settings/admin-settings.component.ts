import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule, FormBuilder, FormGroup, Validators } from '@angular/forms';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatInputModule } from '@angular/material/input';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatSlideToggleModule } from '@angular/material/slide-toggle';
import { MatSelectModule } from '@angular/material/select';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';
import { MatTabsModule } from '@angular/material/tabs';
import { ApiService } from '../../../core/services/api.service';

interface SystemSettings {
  apiUrl: string;
  maxConnections: number;
  sessionTimeout: number;
  enableLogging: boolean;
  enableMetrics: boolean;
  alertEmail: string;
  maintenanceMode: boolean;
}

@Component({
  selector: 'app-admin-settings',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    MatCardModule,
    MatButtonModule,
    MatInputModule,
    MatFormFieldModule,
    MatSlideToggleModule,
    MatSelectModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatSnackBarModule,
    MatTabsModule
  ],
  template: `
    <div class="admin-settings">
      <div class="header">
        <h2>System Settings</h2>
      </div>

      <mat-tab-group>
        <!-- General Settings -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>settings</mat-icon>
            <span>General</span>
          </ng-template>

          <div class="tab-content">
            <mat-card class="settings-card">
              <mat-card-header>
                <mat-card-title>General Configuration</mat-card-title>
              </mat-card-header>

              <mat-card-content [formGroup]="generalSettingsForm">
                <div class="form-field">
                  <mat-form-field>
                    <mat-label>API URL</mat-label>
                    <input matInput formControlName="apiUrl" type="url">
                    <mat-icon matSuffix>link</mat-icon>
                  </mat-form-field>
                </div>

                <div class="form-field">
                  <mat-form-field>
                    <mat-label>Max Connections Per User</mat-label>
                    <input matInput formControlName="maxConnections" type="number" min="1" max="100">
                    <mat-icon matSuffix>people</mat-icon>
                  </mat-form-field>
                </div>

                <div class="form-field">
                  <mat-form-field>
                    <mat-label>Session Timeout (minutes)</mat-label>
                    <input matInput formControlName="sessionTimeout" type="number" min="5" max="1440">
                    <mat-icon matSuffix>schedule</mat-icon>
                  </mat-form-field>
                </div>

                <div class="form-field">
                  <mat-form-field>
                    <mat-label>Alert Email</mat-label>
                    <input matInput formControlName="alertEmail" type="email">
                    <mat-icon matSuffix>email</mat-icon>
                  </mat-form-field>
                </div>
              </mat-card-content>

              <mat-card-actions>
                <button mat-raised-button color="primary" (click)="saveGeneralSettings()">
                  <mat-icon>save</mat-icon> Save Changes
                </button>
                <button mat-button (click)="resetGeneralSettings()">Reset</button>
              </mat-card-actions>
            </mat-card>
          </div>
        </mat-tab>

        <!-- Feature Toggles -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>toggle_on</mat-icon>
            <span>Features</span>
          </ng-template>

          <div class="tab-content">
            <div class="toggles-grid">
              <mat-card class="toggle-card">
                <mat-card-header>
                  <mat-card-title>Enable Logging</mat-card-title>
                  <mat-slide-toggle [checked]="settings.enableLogging" 
                                    (change)="toggleSetting('enableLogging')">
                  </mat-slide-toggle>
                </mat-card-header>
                <mat-card-content>
                  Log all system and user activities. Required for audit trails and debugging.
                </mat-card-content>
              </mat-card>

              <mat-card class="toggle-card">
                <mat-card-header>
                  <mat-card-title>Enable Metrics Collection</mat-card-title>
                  <mat-slide-toggle [checked]="settings.enableMetrics" 
                                    (change)="toggleSetting('enableMetrics')">
                  </mat-slide-toggle>
                </mat-card-header>
                <mat-card-content>
                  Collect performance metrics for monitoring and analytics dashboards.
                </mat-card-content>
              </mat-card>

              <mat-card class="toggle-card">
                <mat-card-header>
                  <mat-card-title>Maintenance Mode</mat-card-title>
                  <mat-slide-toggle [checked]="settings.maintenanceMode" 
                                    (change)="toggleMaintenanceMode()">
                  </mat-slide-toggle>
                </mat-card-header>
                <mat-card-content>
                  <strong>⚠️ Restricts access to admin users only.</strong> Enable during system maintenance.
                </mat-card-content>
              </mat-card>
            </div>
          </div>
        </mat-tab>

        <!-- API Configuration -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>api</mat-icon>
            <span>API</span>
          </ng-template>

          <div class="tab-content">
            <mat-card class="settings-card">
              <mat-card-header>
                <mat-card-title>API Configuration</mat-card-title>
              </mat-card-header>

              <mat-card-content>
                <div class="info-section">
                  <h3>Current API Status</h3>
                  <div class="status-grid">
                    <div class="status-item">
                      <span class="label">API Version</span>
                      <span class="value">v1.2.0</span>
                    </div>
                    <div class="status-item">
                      <span class="label">Endpoints</span>
                      <span class="value">13 active</span>
                    </div>
                    <div class="status-item">
                      <span class="label">Rate Limiting</span>
                      <span class="value">1000 req/min</span>
                    </div>
                    <div class="status-item">
                      <span class="label">Authentication</span>
                      <span class="value badge">JWT + OAuth2</span>
                    </div>
                  </div>
                </div>

                <div class="info-section">
                  <h3>API Keys Management</h3>
                  <table class="api-keys-table">
                    <tr>
                      <th>Key ID</th>
                      <th>Created</th>
                      <th>Last Used</th>
                      <th>Actions</th>
                    </tr>
                    <tr *ngFor="let key of apiKeys">
                      <td>{{ key.id }}</td>
                      <td>{{ key.created }}</td>
                      <td>{{ key.lastUsed }}</td>
                      <td>
                        <button mat-icon-button color="primary" title="Regenerate">
                          <mat-icon>refresh</mat-icon>
                        </button>
                        <button mat-icon-button color="warn" title="Revoke">
                          <mat-icon>delete</mat-icon>
                        </button>
                      </td>
                    </tr>
                  </table>
                  <button mat-raised-button color="primary" style="margin-top: 1rem;">
                    <mat-icon>add</mat-icon> Generate New Key
                  </button>
                </div>
              </mat-card-content>
            </mat-card>
          </div>
        </mat-tab>

        <!-- Security Settings -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>security</mat-icon>
            <span>Security</span>
          </ng-template>

          <div class="tab-content">
            <div class="security-section">
              <mat-card class="settings-card">
                <mat-card-header>
                  <mat-card-title>Security Configuration</mat-card-title>
                </mat-card-header>

                <mat-card-content>
                  <div class="security-item">
                    <div class="security-header">
                      <h3>Two-Factor Authentication</h3>
                      <mat-slide-toggle>2FA Enabled</mat-slide-toggle>
                    </div>
                    <p class="description">Requires administrators to use 2FA for account security.</p>
                  </div>

                  <div class="security-item">
                    <div class="security-header">
                      <h3>IP Whitelist</h3>
                      <button mat-button>Configure</button>
                    </div>
                    <p class="description">Restrict admin access to specific IP addresses.</p>
                  </div>

                  <div class="security-item">
                    <div class="security-header">
                      <h3>SSL/TLS Configuration</h3>
                      <button mat-button>Manage Certificates</button>
                    </div>
                    <p class="description">Current certificate valid until: 2025-12-31</p>
                  </div>

                  <div class="security-item">
                    <div class="security-header">
                      <h3>Password Policy</h3>
                      <button mat-button>Edit Policy</button>
                    </div>
                    <p class="description">Minimum length: 12 | Require special chars: Yes | Expiry: 90 days</p>
                  </div>

                  <div class="security-item">
                    <div class="security-header">
                      <h3>Audit Logs</h3>
                      <button mat-button (click)="viewAuditLogs()">View Logs</button>
                    </div>
                    <p class="description">Track all administrative actions and changes.</p>
                  </div>
                </mat-card-content>
              </mat-card>
            </div>
          </div>
        </mat-tab>

        <!-- Backup & Recovery -->
        <mat-tab>
          <ng-template mat-tab-label>
            <mat-icon>backup</mat-icon>
            <span>Backup</span>
          </ng-template>

          <div class="tab-content">
            <mat-card class="settings-card">
              <mat-card-header>
                <mat-card-title>Backup & Recovery</mat-card-title>
              </mat-card-header>

              <mat-card-content>
                <div class="backup-info">
                  <p><strong>Last Backup:</strong> 2 hours ago</p>
                  <p><strong>Backup Schedule:</strong> Every 6 hours</p>
                  <p><strong>Retention Policy:</strong> 30 days</p>
                </div>

                <h3>Backup Management</h3>
                <div class="backup-actions">
                  <button mat-raised-button color="primary">
                    <mat-icon>backup</mat-icon> Create Backup Now
                  </button>
                  <button mat-raised-button>
                    <mat-icon>storage</mat-icon> View Backups
                  </button>
                  <button mat-raised-button>
                    <mat-icon>restore</mat-icon> Restore from Backup
                  </button>
                  <button mat-raised-button color="warn" (click)="testDisasterRecovery()">
                    <mat-icon>warning</mat-icon> Test DR Plan
                  </button>
                </div>
              </mat-card-content>
            </mat-card>
          </div>
        </mat-tab>
      </mat-tab-group>
    </div>
  `,
  styles: [`
    .admin-settings {
      padding: 2rem;
    }

    .header {
      margin-bottom: 2rem;
    }

    .header h2 {
      margin: 0;
      color: #333;
    }

    .tab-content {
      padding: 2rem;
    }

    .settings-card {
      margin-bottom: 2rem;
    }

    .form-field {
      margin-bottom: 1.5rem;
    }

    mat-form-field {
      width: 100%;
      max-width: 500px;
    }

    .toggles-grid {
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
      gap: 2rem;
    }

    .toggle-card {
      position: relative;
    }

    .toggle-card mat-slide-toggle {
      position: absolute;
      top: 1rem;
      right: 1rem;
    }

    .toggle-card mat-card-header {
      margin-bottom: 1rem;
      display: flex;
      justify-content: space-between;
      align-items: flex-start;
    }

    .toggle-card mat-card-content {
      margin-top: 1rem;
      font-size: 0.9rem;
      color: #666;
    }

    .info-section {
      margin-bottom: 2rem;
    }

    .info-section h3 {
      margin-top: 0;
      color: #333;
      border-bottom: 2px solid #2196f3;
      padding-bottom: 0.5rem;
    }

    .status-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
      gap: 1rem;
    }

    .status-item {
      display: flex;
      flex-direction: column;
      padding: 1rem;
      background: #f5f5f5;
      border-radius: 4px;
    }

    .status-item .label {
      font-size: 0.8rem;
      color: #999;
      text-transform: uppercase;
      font-weight: 500;
    }

    .status-item .value {
      font-size: 1rem;
      font-weight: bold;
      color: #333;
      margin-top: 0.5rem;
    }

    .badge {
      background: #2196f3;
      color: white;
      padding: 0.25rem 0.5rem;
      border-radius: 4px;
    }

    .api-keys-table {
      width: 100%;
      border-collapse: collapse;
      margin: 1rem 0;
      background: white;
    }

    .api-keys-table th {
      background: #2196f3;
      color: white;
      padding: 1rem;
      text-align: left;
      font-weight: 500;
    }

    .api-keys-table td {
      padding: 1rem;
      border-bottom: 1px solid #e0e0e0;
    }

    .security-section {
      display: grid;
      gap: 1.5rem;
    }

    .security-item {
      padding: 1.5rem;
      background: #f5f5f5;
      border-radius: 4px;
      border-left: 4px solid #2196f3;
    }

    .security-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 0.5rem;
    }

    .security-header h3 {
      margin: 0;
      color: #333;
    }

    .description {
      margin: 0;
      color: #666;
      font-size: 0.9rem;
    }

    .backup-info {
      background: #e3f2fd;
      padding: 1rem;
      border-radius: 4px;
      margin-bottom: 2rem;
    }

    .backup-info p {
      margin: 0.5rem 0;
      font-size: 0.95rem;
    }

    .backup-actions {
      display: flex;
      gap: 1rem;
      flex-wrap: wrap;
    }

    mat-tab-group {
      background: white;
      border-radius: 4px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }
  `]
})
export class AdminSettingsComponent implements OnInit {
  private apiService = inject(ApiService);
  private snackBar = inject(MatSnackBar);
  private fb = inject(FormBuilder);

  generalSettingsForm: FormGroup;

  settings: SystemSettings = {
    apiUrl: 'https://api.vpnservice.com',
    maxConnections: 5,
    sessionTimeout: 120,
    enableLogging: true,
    enableMetrics: true,
    alertEmail: 'admin@vpnservice.com',
    maintenanceMode: false
  };

  apiKeys = [
    { id: 'key_prod_abc123...', created: '2024-01-15', lastUsed: '2 hours ago' },
    { id: 'key_staging_def456...', created: '2024-02-01', lastUsed: '1 day ago' },
    { id: 'key_dev_ghi789...', created: '2024-02-10', lastUsed: '30 minutes ago' }
  ];

  ngOnInit() {
    this.initializeForm();
  }

  initializeForm() {
    this.generalSettingsForm = this.fb.group({
      apiUrl: [this.settings.apiUrl, Validators.required],
      maxConnections: [this.settings.maxConnections, [Validators.required, Validators.min(1)]],
      sessionTimeout: [this.settings.sessionTimeout, [Validators.required, Validators.min(5)]],
      alertEmail: [this.settings.alertEmail, [Validators.required, Validators.email]]
    });
  }

  saveGeneralSettings() {
    if (this.generalSettingsForm.valid) {
      Object.assign(this.settings, this.generalSettingsForm.value);
      this.snackBar.open('Settings saved successfully', 'Close', { duration: 3000 });
    }
  }

  resetGeneralSettings() {
    this.initializeForm();
  }

  toggleSetting(settingKey: keyof SystemSettings) {
    this.settings[settingKey] = !this.settings[settingKey];
    this.snackBar.open(`Setting updated: ${settingKey}`, 'Close', { duration: 2000 });
  }

  toggleMaintenanceMode() {
    this.settings.maintenanceMode = !this.settings.maintenanceMode;
    this.snackBar.open(
      `Maintenance mode ${this.settings.maintenanceMode ? 'enabled' : 'disabled'}`,
      'Close',
      { duration: 3000 }
    );
  }

  viewAuditLogs() {
    this.snackBar.open('Audit logs would open in a new view', 'Close', { duration: 3000 });
  }

  testDisasterRecovery() {
    if (confirm('This will test the disaster recovery procedure. Continue?')) {
      this.snackBar.open('DR test initiated. Check alerts for updates.', 'Close', { duration: 5000 });
    }
  }
}
