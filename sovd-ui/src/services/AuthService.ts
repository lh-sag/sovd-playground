import { ref } from 'vue';
import { parseJWT } from '../util/jwt';
import { notificationService } from './NotificationService';
import { Level, telemetryService } from './TelemetryService';

export class AuthService {
    TOKEN_KEY = 'sovd_auth_token';
    TOKEN_EXPIRY_WARNING = 5 * 60 * 1000;
    TOKEN_CHECK_INTERVAL = 5 * 60 * 1000;

    token = ref<string | null>(null);
    claims = ref<Record<string, any> | null>(null);
    tokenExpiryWarning = ref(false);

    tokenExpiryCheckInterval: ReturnType<typeof setInterval> | null = null;

    init() {
        const token = this.loadTokenFromStorage();
        if (!token) return;

        const claims = parseJWT(token);
        if (claims && !this.isTokenExpired(claims)) {
            this.token.value = token;
            this.claims.value = claims;
            this.startTokenExpiryCheck();
        } else {
            this.clearTokenFromStorage();
        }
    }

    deinit() {
        this.stopTokenExpiryCheck();
    }

    login(token: string, persistToken: boolean) {
        const claims = parseJWT(token);
        if (!claims) {
            notificationService.showToast('Invalid Token', 'The provided token is not a valid JWT', Level.ERROR);
            return;
        }

        if (this.isTokenExpired(claims)) {
            notificationService.showToast('Token Expired', 'The provided token has already expired', Level.ERROR);
            return;
        }

        this.token.value = token;
        this.claims.value = claims;

        this.saveTokenToStorage(this.token.value, persistToken);
        this.startTokenExpiryCheck();

        telemetryService.log('Authentication Success', `Authenticated as ${claims.sub || 'User'}`);
    }

    logout() {
        this.clearTokenFromStorage();
        this.token.value = null;
        this.claims.value = null;
        this.tokenExpiryWarning.value = false;
        this.stopTokenExpiryCheck();

        telemetryService.log('Logout', 'User logged out');
    }

    getAuthHeader(): RequestInit['headers'] {
        if (!this.token.value) return {};
        return { Authorization: `Bearer ${this.token.value}` };
    }

    private loadTokenFromStorage() {
        const usePersistent = localStorage.getItem(`${this.TOKEN_KEY}_persist`) === 'true';
        const storage = usePersistent ? localStorage : sessionStorage;
        return storage.getItem(this.TOKEN_KEY);
    }

    private saveTokenToStorage(token: string, persistToken: boolean) {
        const storage = persistToken ? localStorage : sessionStorage;
        storage.setItem(this.TOKEN_KEY, token);
        if (persistToken) {
            localStorage.setItem(`${this.TOKEN_KEY}_persist`, 'true');
        } else {
            localStorage.removeItem(`${this.TOKEN_KEY}_persist`);
        }
    }

    private clearTokenFromStorage() {
        sessionStorage.removeItem(this.TOKEN_KEY);
        localStorage.removeItem(this.TOKEN_KEY);
        localStorage.removeItem(`${this.TOKEN_KEY}_persist`);
    }

    private isTokenExpired(claims: Record<string, any>) {
        if (!claims?.exp) return false;
        const now = Math.floor(Date.now() / 1000);
        return now >= claims.exp;
    }

    private isTokenExpiringSoon(claims: Record<string, any>) {
        if (!claims?.exp) return false;
        const expiryTime = claims.exp * 1000;
        const timeUntilExpiry = expiryTime - Date.now();
        return timeUntilExpiry > 0 && timeUntilExpiry <= this.TOKEN_EXPIRY_WARNING;
    }

    private startTokenExpiryCheck() {
        this.stopTokenExpiryCheck();
        this.tokenExpiryCheckInterval = setInterval(() => {
            if (!this.claims.value) return;

            if (this.isTokenExpired(this.claims.value)) {
                notificationService.showToast(
                    'Token Expired',
                    'Your authentication token has expired. Please sign in again.',
                    Level.WARNING,
                );
                this.logout();
            } else {
                this.tokenExpiryWarning.value = this.isTokenExpiringSoon(this.claims.value);
            }
        }, this.TOKEN_CHECK_INTERVAL);
    }

    private stopTokenExpiryCheck() {
        if (this.tokenExpiryCheckInterval) {
            clearInterval(this.tokenExpiryCheckInterval);
            this.tokenExpiryCheckInterval = null;
        }
    }
}

export const authService = new AuthService();
