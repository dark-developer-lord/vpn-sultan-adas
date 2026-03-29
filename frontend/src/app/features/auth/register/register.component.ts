import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormBuilder, FormGroup, ReactiveFormsModule, Validators } from '@angular/forms';
import { Router, RouterModule } from '@angular/router';
import { AuthService } from '@app/core/services/auth.service';
import { MatCardModule } from '@angular/material/card';
import { MatFormFieldModule } from '@angular/material/form-field';
import { MatInputModule } from '@angular/material/input';
import { MatButtonModule } from '@angular/material/button';
import { MatProgressSpinnerModule } from '@angular/material/progress-spinner';
import { MatSnackBar, MatSnackBarModule } from '@angular/material/snack-bar';

@Component({
  selector: 'app-register',
  standalone: true,
  imports: [
    CommonModule,
    ReactiveFormsModule,
    RouterModule,
    MatCardModule,
    MatFormFieldModule,
    MatInputModule,
    MatButtonModule,
    MatProgressSpinnerModule,
    MatSnackBarModule
  ],
  template: `
    <div class="register-container">
      <mat-card class="register-card">
        <mat-card-header>
          <mat-card-title>Create Account</mat-card-title>
          <mat-card-subtitle>Join VPN Sultan Adas</mat-card-subtitle>
        </mat-card-header>
        
        <mat-card-content>
          <form [formGroup]="registerForm" (ngSubmit)="onSubmit()">
            <mat-form-field appearance="fill" class="full-width">
              <mat-label>Email</mat-label>
              <input matInput 
                     formControlName="email" 
                     type="email" 
                     placeholder="your@email.com">
              <mat-error *ngIf="email.hasError('required')">Email is required</mat-error>
              <mat-error *ngIf="email.hasError('email')">Please enter a valid email</mat-error>
            </mat-form-field>

            <mat-form-field appearance="fill" class="full-width">
              <mat-label>Password</mat-label>
              <input matInput 
                     formControlName="password" 
                     type="password"
                     placeholder="At least 6 characters">
              <mat-error *ngIf="password.hasError('required')">Password is required</mat-error>
              <mat-error *ngIf="password.hasError('minlength')">Password must be at least 6 characters</mat-error>
            </mat-form-field>

            <mat-form-field appearance="fill" class="full-width">
              <mat-label>Confirm Password</mat-label>
              <input matInput 
                     formControlName="confirmPassword" 
                     type="password"
                     placeholder="Repeat password">
              <mat-error *ngIf="confirmPassword.hasError('required')">Please confirm password</mat-error>
            </mat-form-field>

            <div *ngIf="error" class="error-message">
              {{ error }}
            </div>

            <button mat-raised-button 
                    color="primary" 
                    type="submit"
                    [disabled]="!registerForm.valid || isLoading"
                    class="full-width">
              <mat-spinner *ngIf="isLoading" diameter="20" class="spinner"></mat-spinner>
              {{ isLoading ? 'Creating account...' : 'Sign Up' }}
            </button>
          </form>
        </mat-card-content>

        <mat-card-footer>
          <p>Already have an account? 
            <a routerLink="/login" class="link">Log in here</a>
          </p>
        </mat-card-footer>
      </mat-card>
    </div>
  `,
  styles: [`
    .register-container {
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      padding: 20px;
    }

    .register-card {
      width: 100%;
      max-width: 450px;
      box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
    }

    mat-card-header {
      text-align: center;
      margin-bottom: 30px;
    }

    mat-card-title {
      font-size: 28px;
      color: #333;
    }

    mat-card-subtitle {
      color: #999;
      margin-top: 10px;
    }

    mat-form-field {
      width: 100%;
      margin-bottom: 20px;
    }

    .full-width {
      width: 100%;
    }

    button {
      margin-top: 10px;
    }

    .error-message {
      color: #f44336;
      padding: 12px;
      background-color: #ffebee;
      border-radius: 4px;
      margin-bottom: 20px;
      text-align: center;
    }

    mat-card-footer {
      text-align: center;
      padding-top: 20px;
      border-top: 1px solid #eee;
      margin-top: 20px;
    }

    .link {
      color: #667eea;
      text-decoration: none;
      font-weight: 500;
      cursor: pointer;
    }

    .link:hover {
      text-decoration: underline;
    }

    .spinner {
      display: inline-block;
      margin-right: 10px;
    }
  `]
})
export class RegisterComponent {
  registerForm: FormGroup;
  isLoading = false;
  error: string | null = null;

  constructor(
    private fb: FormBuilder,
    private auth: AuthService,
    private router: Router,
    private snackBar: MatSnackBar
  ) {
    this.registerForm = this.fb.group({
      email: ['', [Validators.required, Validators.email]],
      password: ['', [Validators.required, Validators.minLength(6)]],
      confirmPassword: ['', [Validators.required]]
    });
  }

  get email() {
    return this.registerForm.get('email')!;
  }

  get password() {
    return this.registerForm.get('password')!;
  }

  get confirmPassword() {
    return this.registerForm.get('confirmPassword')!;
  }

  onSubmit(): void {
    if (!this.registerForm.valid) {
      this.error = 'Please fill in all fields correctly';
      return;
    }

    const { email, password, confirmPassword } = this.registerForm.value;

    if (password !== confirmPassword) {
      this.error = 'Passwords do not match';
      return;
    }

    this.isLoading = true;
    this.error = null;

    this.auth.register(email, password).subscribe({
      next: (response) => {
        this.snackBar.open('Registration successful! Redirecting...', 'Close', { duration: 3000 });
        setTimeout(() => {
          this.router.navigate(['/dashboard']);
        }, 1000);
      },
      error: (error) => {
        this.isLoading = false;
        this.error = error.message || 'Registration failed. Please try again.';
        console.error('Registration error:', error);
      }
    });
  }
}
