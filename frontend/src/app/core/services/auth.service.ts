import { Injectable } from '@angular/core';
import { ApiService } from './api.service';
import { BehaviorSubject, Observable } from 'rxjs';
import { tap } from 'rxjs/operators';

@Injectable({
  providedIn: 'root'
})
export class AuthService {
  private isAuthenticatedSubject = new BehaviorSubject<boolean>(this.hasToken());
  public isAuthenticated$ = this.isAuthenticatedSubject.asObservable();

  constructor(private api: ApiService) {}

  private hasToken(): boolean {
    return !!localStorage.getItem('token');
  }

  register(email: string, password: string): Observable<any> {
    return this.api.register(email, password).pipe(
      tap((response) => {
        // API returns token directly in response
        const token = response.token || response.data?.token;
        if (token) {
          this.storeToken(token);
          this.isAuthenticatedSubject.next(true);
        }
      })
    );
  }

  login(email: string, password: string): Observable<any> {
    return this.api.login(email, password).pipe(
      tap((response) => {
        // API returns token directly in response
        const token = response.token || response.data?.token;
        if (token) {
          this.storeToken(token);
          this.isAuthenticatedSubject.next(true);
        }
      })
    );
  }

  logout(): Observable<any> {
    return new Observable((observer) => {
      this.clearToken();
      this.isAuthenticatedSubject.next(false);
      observer.next(true);
      observer.complete();
    });
  }

  private storeToken(token: string): void {
    localStorage.setItem('token', token);
  }

  private clearToken(): void {
    localStorage.removeItem('token');
  }

  getToken(): string | null {
    return localStorage.getItem('token');
  }

  isAuthenticated(): boolean {
    return this.hasToken();
  }
}
