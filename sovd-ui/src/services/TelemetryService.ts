import { ref } from 'vue';
import { CONSTANTS } from '../constants';
import { authService } from './AuthService';
import { notificationService } from './NotificationService';

export enum Level {
    INFO = 'info',
    WARNING = 'warning',
    ERROR = 'error',
}

interface Metrics {
    apiCalls: ApiCallMetric[];
    errors: ErrorMetric[];
    interactions: InteractionMetric[];
    performance: PerformanceMetric[];
}

interface ApiCallMetric {
    url: string;
    method: string;
    timestamp: string;
    status: number | 'error' | 'unknown';
    error?: string;
    authenticated: boolean;
    duration: number;
}

interface ErrorMetric {}

interface InteractionMetric {}

interface PerformanceMetric {}

export class TelemetryService {
    metrics: Metrics = {
        apiCalls: [],
        errors: [],
        interactions: [],
        performance: [],
    };
    lastMetricsHash: string | null = null;

    authenticationFailureDetected = ref(false);

    init() {
        globalThis.addEventListener('error', (event) => {
            this.logError('Global Error', {
                message: event.message,
                source: event.filename,
                line: event.lineno,
                column: event.colno,
                error: event.error,
            });
        });

        globalThis.addEventListener('unhandledrejection', (event) => {
            this.logError('Unhandled Promise Rejection', {
                reason: event.reason,
                promise: event.promise,
            });
        });
    }

    async trackApiCall(
        url: string,
        method = 'GET',
        options: {
            headers?: Record<string, string>;
            body?: unknown;
        } = {},
    ) {
        const { headers = {}, body = null } = options;

        const startTime = performance.now();

        // Build default headers
        const defaultHeaders = {
            'Content-Type': 'application/json',
            ...authService.getAuthHeader(),
            ...headers,
        };

        // Build fetch configuration
        const fetchConfig: RequestInit = {
            method,
            headers: defaultHeaders,
        };

        if (body && method !== 'GET') {
            fetchConfig.body = typeof body === 'string' ? body : JSON.stringify(body);
        }

        try {
            const response = await fetch(url, fetchConfig);
            const duration = performance.now() - startTime;

            this.addMetric(this.metrics.apiCalls, {
                url,
                method,
                timestamp: new Date().toISOString(),
                duration,
                status: response?.status ?? 'unknown',
                authenticated: !!authService.token.value,
            });

            if (duration > CONSTANTS.SLOW_API_THRESHOLD) {
                this.log('Slow API Call', `${method} ${url} took ${duration.toFixed(0)}ms`, Level.WARNING);
            }

            if (!response.ok) {
                this.logError(
                    `API Error: ${response.status} ${response.statusText}`,
                    { url, status: response.status },
                    { showToast: false },
                );
            }

            if (response.status === 401) {
                this.log('Authentication Required', `${method} ${url} requires authentication`, Level.WARNING);
                if (authService.token.value) {
                    notificationService.showToast(
                        'Authentication Failed',
                        'Your token may have expired. Please re-authenticate.',
                        Level.WARNING,
                    );
                }
                this.authenticationFailureDetected.value = true;
            }

            return response;
        } catch (error) {
            const message = error instanceof Error ? error.message : String(error);
            this.addMetric(this.metrics.apiCalls, {
                url,
                method,
                timestamp: new Date().toISOString(),
                duration: performance.now() - startTime,
                status: 'error',
                error: message,
                authenticated: !!authService.token.value,
            });
            this.logError('API Call Failed', { url, method, error: message }, { showToast: false });

            throw error;
        }
    }

    trackInteraction(action: string, details: unknown = {}) {
        const interaction = {
            action,
            details,
            timestamp: new Date().toISOString(),
            viewport: {
                width: window.innerWidth,
                height: window.innerHeight,
            },
            userAgent: navigator.userAgent,
        };

        this.addMetric(this.metrics.interactions, interaction);
    }

    logError(
        context: string,
        details: unknown = {},
        options: { consoleLevel?: 'error' | 'warn' | 'log' | false; showToast?: boolean; toastMessage?: string } = {},
    ) {
        const {
            consoleLevel = 'error', // 'error', 'warn', 'log', or false
            showToast = true,
            toastMessage = null,
        } = options;

        const error = {
            context,
            details,
            timestamp: new Date().toISOString(),
            stack: new Error('stack').stack,
        };

        this.addMetric(this.metrics.errors, error);

        // Flexible console output
        if (consoleLevel) {
            const consoleMsg =
                typeof details === 'object' ? `[Telemetry] ${context}:` : `[Telemetry] ${context}: ${details}`;

            console[consoleLevel](consoleMsg, typeof details === 'object' ? details : '');
        }

        // Show toast with custom or default message
        if (showToast) {
            const message = toastMessage || 'An error occurred. Check console for details.';
            notificationService.showToast(context, message, Level.ERROR);
        }
    }

    log(title: string, message: string, level = Level.INFO) {
        switch (level) {
            case Level.ERROR:
                console.error(`[Telemetry] ${title}:`, message);
                break;
            case Level.WARNING:
                console.warn(`[Telemetry] ${title}:`, message);
                break;
            default:
                console.log(`[Telemetry] ${title}:`, message);
        }
    }

    getSummary() {
        const apiCalls = this.metrics.apiCalls;
        const totalDuration = apiCalls.reduce((sum, c) => sum + (c.duration || 0), 0);

        return {
            apiCalls: {
                total: apiCalls.length,
                failed: apiCalls.filter((c) => c.error).length,
                averageDuration: apiCalls.length ? totalDuration / apiCalls.length : 0,
            },
            errors: this.metrics.errors.length,
            interactions: this.metrics.interactions.length,
            uptime: performance.now(),
        };
    }

    addMetric<T>(metrics: T[], metric: T) {
        if (metrics.length >= CONSTANTS.MAX_METRICS_SIZE) {
            const retentionSize = Math.floor(CONSTANTS.MAX_METRICS_SIZE * 0.8);
            metrics.splice(0, retentionSize);
        }
        metrics.push(metric);
    }

    getMetricsHash(): string {
        const summary = this.getSummary();
        return JSON.stringify(summary);
    }

    shouldLogSummary() {
        const currentHash = this.getMetricsHash();
        if (currentHash !== this.lastMetricsHash) {
            this.lastMetricsHash = currentHash;
            return true;
        }
        return false;
    }

    cleanup() {
        this.metrics = {
            apiCalls: [],
            errors: [],
            interactions: [],
            performance: [],
        };
        this.lastMetricsHash = null;
        this.authenticationFailureDetected.value = false;
    }

    exportMetrics() {
        return JSON.stringify(this.metrics, null, 2);
    }
}

export const telemetryService = new TelemetryService();
