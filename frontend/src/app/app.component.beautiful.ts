import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterOutlet, Router } from '@angular/router';
import { MatToolbarModule } from '@angular/material/toolbar';
import { MatSidenavModule } from '@angular/material/sidenav';
import { MatListModule } from '@angular/material/list';
import { MatIconModule } from '@angular/material/icon';
import { MatButtonModule } from '@angular/material/button';
import { AuthService } from '@app/core/services/auth.service';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [
    CommonModule, 
    RouterOutlet,
    MatToolbarModule,
    MatSidenavModule,
    MatListModule,
    MatIconModule,
    MatButtonModule
  ],
  template: `
    <mat-toolbar color="primary" class="main-toolbar">
      <button mat-icon-button (click)="sidenav.toggle()" class="menu-button">
        <mat-icon>menu</mat-icon>
      </button>
      <span class="toolbar-title">🔐 VPN Sultan Adas</span>
      <span class="spacer"></span>
      <button mat-icon-button (click)="logout()" *ngIf="isLoggedIn" title="Logout">
        <mat-icon>logout</mat-icon>
      </button>
    </mat-toolbar>

    <mat-sidenav-container class="sidenav-container" *ngIf="isLoggedIn">
      <mat-sidenav #sidenav class="sidenav" mode="side" [opened]="true" [fixedInViewport]="false">
        <mat-nav-list>
          <mat-list-item routerLink="/dashboard" (click)="sidenav.close()">
            <mat-icon matListItemIcon>dashboard</mat-icon>
            <span matListItemTitle>Dashboard</span>
          </mat-list-item>
          <mat-list-item routerLink="/peers" (click)="sidenav.close()">
            <mat-icon matListItemIcon>router</mat-icon>
            <span matListItemTitle>My Peers</span>
          </mat-list-item>
          <mat-list-item routerLink="/nodes" (click)="sidenav.close()">
            <mat-icon matListItemIcon>cloud_queue</mat-icon>
            <span matListItemTitle>VPN Nodes</span>
          </mat-list-item>
          <mat-divider></mat-divider>
          <mat-list-item (click)="logout()">
            <mat-icon matListItemIcon color="warn">power_settings_new</mat-icon>
            <span matListItemTitle>Logout</span>
          </mat-list-item>
        </mat-nav-list>
      </mat-sidenav>

      <mat-sidenav-content class="main-content">
        <router-outlet></router-outlet>
      </mat-sidenav-content>
    </mat-sidenav-container>

    <div class="main-content" *ngIf="!isLoggedIn">
      <router-outlet></router-outlet>
    </div>
  `,
  styles: [`
    ::ng-deep {
      body {
        margin: 0;
        font-family: 'Roboto', sans-serif;
      }
    }

    .main-toolbar {
      box-shadow: 0 2px 4px rgba(0,0,0,0.1);
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
    }

    .toolbar-title {
      font-weight: 600;
      font-size: 18px;
      margin-left: 16px;
    }

    .spacer {
      flex: 1 1 auto;
    }

    .menu-button {
      @media (min-width: 769px) {
        display: none;
      }
    }

    .sidenav-container {
      height: calc(100vh - 64px);
    }

    .sidenav {
      width: 250px;
      background: #f5f5f5;
    }

    .main-content {
      padding: 24px;
      background: #fafafa;
      min-height: calc(100vh - 64px);
      overflow-y: auto;
    }

    @media (max-width: 768px) {
      .sidenav {
        width: 200px;
      }
    }
  `]
})
export class AppComponent implements OnInit {
  title = 'VPN Dashboard';
  isLoggedIn = false;

  constructor(private authService: AuthService, private router: Router) {}

  ngOnInit() {
    this.isLoggedIn = this.authService.isAuthenticated();
  }

  logout() {
    this.authService.logout();
    this.isLoggedIn = false;
    this.router.navigate(['/login']);
  }
}
