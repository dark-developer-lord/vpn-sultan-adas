import { Component, OnInit, inject } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule, ReactiveFormsModule } from '@angular/forms';
import { MatTabsModule } from '@angular/material/tabs';
import { MatCardModule } from '@angular/material/card';
import { MatButtonModule } from '@angular/material/button';
import { MatIconModule } from '@angular/material/icon';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatDatepickerModule } from '@angular/material/datepicker';
import { MatNativeDateModule } from '@angular/material/core';
import { MatFormFieldModule } from '@angular/material/form-field';
import { ApiService } from '../../../core/services/api.service';

interface AnalyticsData {
  period: string;
  users: number;
  connections: number;
  dataUsage: number;
  revenue: number;
  uptime: number;
  avgLatency: number;
}

interface ChartData {
  label: string;
  value: number;
  percentage: number;
}

@Component({
  selector: 'app-admin-analytics',
  standalone: true,
  imports: [
    CommonModule,
    FormsModule,
    ReactiveFormsModule,
    MatTabsModule,
    MatCardModule,
    MatButtonModule,
    MatIconModule,
    MatProgressSpinnerModule,
    MatDatepickerModule,
    MatNativeDateModule,
    MatFormFieldModule
  ],
  template: `
    <div class="admin-analytics">
      <div class="header">
        <h2>Analytics & Reporting</h2>
        <div class="date-range">
          <button mat-button>
            <mat-icon>calendar_today</mat-icon> Last 30 Days
          </button>
          <button mat-button>Export</button>
        </div>
      </div>

      <div *ngIf="isLoading" class="spinner">
        <mat-spinner diameter="50"></mat-spinner>
      </div>

      <div *ngIf="!isLoading">
        <!-- Key Metrics -->
        <div class="metrics-grid">
          <div class="metric-card">
            <div class="metric-header">
              <h3>Total Users</h3>
              <mat-icon>people</mat-icon>
            </div>
            <p class="metric-value">{{ currentStats.users.toLocaleString() }}</p>
            <p class="metric-change positive">↑ 12% from last month</p>
          </div>

          <div class="metric-card">
            <div class="metric-header">
              <h3>Active Connections</h3>
              <mat-icon>cloud_queue</mat-icon>
            </div>
            <p class="metric-value">{{ currentStats.connections.toLocaleString() }}</p>
            <p class="metric-change positive">↑ 8.5% from last month</p>
          </div>

          <div class="metric-card">
            <div class="metric-header">
              <h3>Data Usage</h3>
              <mat-icon>data_usage</mat-icon>
            </div>
            <p class="metric-value">{{ currentStats.dataUsage }}TB</p>
            <p class="metric-change negative">↓ 3.2% from last month</p>
          </div>

          <div class="metric-card">
            <div class="metric-header">
              <h3>Revenue</h3>
              <mat-icon>attach_money</mat-icon>
            </div>
            <p class="metric-value">${{ currentStats.revenue.toLocaleString() }}</p>
            <p class="metric-change positive">↑ 15% from last month</p>
          </div>

          <div class="metric-card">
            <div class="metric-header">
              <h3>Uptime</h3>
              <mat-icon>trending_up</mat-icon>
            </div>
            <p class="metric-value">{{ currentStats.uptime }}%</p>
            <p class="metric-change">SLA: 99.9%</p>
          </div>

          <div class="metric-card">
            <div class="metric-header">
              <h3>Avg Latency</h3>
              <mat-icon>speed</mat-icon>
            </div>
            <p class="metric-value">{{ currentStats.avgLatency }}ms</p>
            <p class="metric-change positive">↓ 5ms from last week</p>
          </div>
        </div>

        <!-- Charts and Breakdowns -->
        <mat-tab-group>
          <mat-tab>
            <ng-template mat-tab-label>
              <mat-icon>bar_chart</mat-icon>
              <span>User Distribution</span>
            </ng-template>

            <div class="tab-content">
              <div class="chart-container">
                <h3>Users by Subscription Plan</h3>
                <div class="breakdown-list">
                  <div class="breakdown-item" *ngFor="let item of userDistribution">
                    <span class="label">{{ item.label }}</span>
                    <div class="bar">
                      <div class="filled" [style.width.%]="item.percentage"></div>
                    </div>
                    <span class="value">{{ item.value.toLocaleString() }} ({{ item.percentage }}%)</span>
                  </div>
                </div>
              </div>
            </div>
          </mat-tab>

          <mat-tab>
            <ng-template mat-tab-label>
              <mat-icon>location_on</mat-icon>
              <span>Geographic Distribution</span>
            </ng-template>

            <div class="tab-content">
              <div class="chart-container">
                <h3>Users by Region</h3>
                <div class="breakdown-list">
                  <div class="breakdown-item" *ngFor="let item of geoDistribution">
                    <span class="label">{{ item.label }}</span>
                    <div class="bar">
                      <div class="filled" [style.width.%]="item.percentage"></div>
                    </div>
                    <span class="value">{{ item.value.toLocaleString() }} ({{ item.percentage }}%)</span>
                  </div>
                </div>
              </div>
            </div>
          </mat-tab>

          <mat-tab>
            <ng-template mat-tab-label>
              <mat-icon>pie_chart</mat-icon>
              <span>Revenue Breakdown</span>
            </ng-template>

            <div class="tab-content">
              <div class="chart-container">
                <h3>Revenue by Plan</h3>
                <div class="breakdown-list">
                  <div class="breakdown-item" *ngFor="let item of revenueBreakdown">
                    <span class="label">{{ item.label }}</span>
                    <div class="bar">
                      <div class="filled" [style.width.%]="item.percentage"></div>
                    </div>
                    <span class="value">${{ item.value.toLocaleString() }} ({{ item.percentage }}%)</span>
                  </div>
                </div>
              </div>
            </div>
          </mat-tab>

          <mat-tab>
            <ng-template mat-tab-label>
              <mat-icon>assessment</mat-icon>
              <span>Performance Metrics</span>
            </ng-template>

            <div class="tab-content">
              <div class="chart-container">
                <h3>Performance Indicators</h3>
                <div class="metrics-table">
                  <div class="table-row header">
                    <span>Metric</span>
                    <span>Current</span>
                    <span>Target</span>
                    <span>Status</span>
                  </div>
                  <div class="table-row">
                    <span>Uptime</span>
                    <span>99.95%</span>
                    <span>99.9%</span>
                    <span class="badge success">✓ Excellent</span>
                  </div>
                  <div class="table-row">
                    <span>P95 Latency</span>
                    <span>125ms</span>
                    <span>200ms</span>
                    <span class="badge success">✓ Good</span>
                  </div>
                  <div class="table-row">
                    <span>Error Rate</span>
                    <span>0.05%</span>
                    <span>0.1%</span>
                    <span class="badge success">✓ Good</span>
                  </div>
                  <div class="table-row">
                    <span>CPU Usage</span>
                    <span>62%</span>
                    <span>80%</span>
                    <span class="badge success">✓ Healthy</span>
                  </div>
                </div>
              </div>
            </div>
          </mat-tab>
        </mat-tab-group>

        <!-- Export Options -->
        <div class="export-section">
          <h3>Export Reports</h3>
          <div class="export-buttons">
            <button mat-raised-button color="primary">
              <mat-icon>picture_as_pdf</mat-icon> Export as PDF
            </button>
            <button mat-raised-button>
              <mat-icon>table_chart</mat-icon> Export as CSV
            </button>
            <button mat-raised-button>
              <mat-icon>email</mat-icon> Email Report
            </button>
          </div>
        </div>
      </div>
    </div>
  `,
  styles: [`
    .admin-analytics {
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

    .date-range {
      display: flex;
      gap: 1rem;
    }

    .metrics-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: 1.5rem;
      margin-bottom: 2rem;
    }

    .metric-card {
      background: white;
      padding: 1.5rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      border-top: 4px solid #2196f3;
    }

    .metric-header {
      display: flex;
      justify-content: space-between;
      align-items: center;
      margin-bottom: 1rem;
    }

    .metric-header h3 {
      margin: 0;
      color: #666;
      font-size: 0.9rem;
      text-transform: uppercase;
      font-weight: 500;
    }

    .metric-header mat-icon {
      color: #2196f3;
      font-size: 1.5rem;
    }

    .metric-value {
      font-size: 2rem;
      font-weight: bold;
      color: #333;
      margin: 0.5rem 0;
    }

    .metric-change {
      font-size: 0.85rem;
      color: #999;
      margin: 0;
    }

    .metric-change.positive {
      color: #4caf50;
    }

    .metric-change.negative {
      color: #f44336;
    }

    .tab-content {
      padding: 2rem;
    }

    .chart-container {
      background: white;
      padding: 2rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .chart-container h3 {
      margin-top: 0;
      color: #333;
    }

    .breakdown-list {
      display: flex;
      flex-direction: column;
      gap: 1rem;
    }

    .breakdown-item {
      display: grid;
      grid-template-columns: 150px 1fr 150px;
      align-items: center;
      gap: 1rem;
    }

    .breakdown-item .label {
      font-weight: 500;
      color: #333;
    }

    .bar {
      height: 24px;
      background: #e0e0e0;
      border-radius: 4px;
      overflow: hidden;
    }

    .bar .filled {
      height: 100%;
      background: linear-gradient(90deg, #2196f3, #1976d2);
    }

    .breakdown-item .value {
      text-align: right;
      font-weight: 500;
      color: #666;
    }

    .metrics-table {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .table-row {
      display: grid;
      grid-template-columns: 1fr 1fr 1fr 1fr;
      gap: 1rem;
      padding: 1rem;
      background: #f5f5f5;
      border-radius: 4px;
    }

    .table-row.header {
      background: #2196f3;
      color: white;
      font-weight: bold;
    }

    .badge {
      padding: 0.25rem 0.75rem;
      border-radius: 12px;
      font-size: 0.85rem;
      font-weight: 500;
      text-align: center;
    }

    .badge.success {
      background: #c8e6c9;
      color: #2e7d32;
    }

    .export-section {
      margin-top: 3rem;
      background: white;
      padding: 2rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    }

    .export-buttons {
      display: flex;
      gap: 1rem;
      margin-top: 1rem;
      flex-wrap: wrap;
    }

    .spinner {
      display: flex;
      justify-content: center;
      padding: 3rem;
    }

    mat-tab-group {
      margin-bottom: 2rem;
    }
  `]
})
export class AdminAnalyticsComponent implements OnInit {
  private apiService = inject(ApiService);

  currentStats: AnalyticsData = {
    period: 'Last 30 Days',
    users: 15234,
    connections: 8567,
    dataUsage: 145,
    revenue: 28500,
    uptime: 99.95,
    avgLatency: 125
  };

  userDistribution: ChartData[] = [
    { label: 'Premium', value: 5234, percentage: 34 },
    { label: 'Professional', value: 7123, percentage: 47 },
    { label: 'Basic', value: 2877, percentage: 19 }
  ];

  geoDistribution: ChartData[] = [
    { label: 'North America', value: 6234, percentage: 41 },
    { label: 'Europe', value: 4567, percentage: 30 },
    { label: 'Asia-Pacific', value: 3123, percentage: 20 },
    { label: 'Other', value: 1310, percentage: 9 }
  ];

  revenueBreakdown: ChartData[] = [
    { label: 'Premium Plan', value: 15234, percentage: 54 },
    { label: 'Professional Plan', value: 8900, percentage: 31 },
    { label: 'Basic Plan', value: 3366, percentage: 12 },
    { label: 'Enterprise', value: 1000, percentage: 3 }
  ];

  isLoading = false;

  ngOnInit() {
    this.loadAnalytics();
  }

  loadAnalytics() {
    // In a real app, this would fetch from the API
    // Simulated data already loaded in component properties
  }
}
