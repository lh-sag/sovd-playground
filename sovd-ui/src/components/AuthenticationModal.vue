<script setup lang="ts">
import { computed, ref } from 'vue';
import { authService } from '../services/AuthService';
import InfoCard from './generic/InfoCard.vue';
import InfoCardRow from './generic/InfoCardRow.vue';

const emit = defineEmits<{
    close: [];
    submit: [token: string, persistToken: boolean];
}>();

const authTokenInput = ref('');
const persistToken = ref(localStorage.getItem(`${authService.TOKEN_KEY}_persist`) === 'true');

function authenticate() {
    authService.login(authTokenInput.value.trim(), persistToken.value);
    authTokenInput.value = '';
}

const tokenExpiryLabel = computed(() => {
    if (!authService.claims.value?.exp) return 'Unknown';

    const now = Math.floor(Date.now() / 1000);
    const diff = authService.claims.value.exp - now;

    try {
        if (typeof Intl === 'undefined' || !Intl.RelativeTimeFormat) {
            if (diff < 0) return 'Expired';
            if (diff < 60) return `${diff}s`;
            if (diff < 3600) return `${Math.floor(diff / 60)}m`;
            if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
            return `${Math.floor(diff / 86400)}d`;
        }

        const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
        if (diff < 0) {
            return 'Expired';
        } else if (diff < 60) {
            return rtf.format(diff, 'second');
        } else if (diff < 3600) {
            return rtf.format(Math.floor(diff / 60), 'minute');
        } else if (diff < 86400) {
            return rtf.format(Math.floor(diff / 3600), 'hour');
        } else {
            return rtf.format(Math.floor(diff / 86400), 'day');
        }
    } catch (error) {
        console.warn('Failed to format token expiry:', error);
        return 'Unknown';
    }
});
</script>

<template>
    <div class="auth-modal" @click.self="emit('close')">
        <div class="auth-modal-content">
            <div class="auth-modal-header">
                <h5>Authentication</h5>
                <button type="button" class="btn-close" @click="emit('close')"></button>
            </div>
            <div class="auth-modal-body">
                <div v-if="!authService.token.value" class="auth-form">
                    <div class="mb-3">
                        <label for="jwtToken" class="form-label">JWT Token</label>
                        <textarea
                            id="jwtToken"
                            v-model="authTokenInput"
                            class="form-control"
                            rows="4"
                            placeholder="Paste your JWT token here..."
                            @keydown.enter.ctrl="authenticate()"
                        ></textarea>
                        <small class="form-text text-muted">
                            Enter your JWT Bearer token to authenticate API requests
                        </small>
                    </div>
                    <div class="mb-3">
                        <div class="form-check">
                            <input type="checkbox" class="form-check-input" id="persistToken" v-model="persistToken" />
                            <label class="form-check-label" for="persistToken">
                                Remember token (stores in localStorage)
                            </label>
                            <small class="form-text text-muted d-block">
                                If unchecked, token will be cleared when you close the tab
                            </small>
                        </div>
                    </div>
                    <div class="d-flex justify-content-end gap-2">
                        <button type="button" class="btn btn-secondary" @click="emit('close')">Cancel</button>
                        <button
                            type="button"
                            class="btn btn-primary"
                            @click="authenticate()"
                            :disabled="!authTokenInput.trim()"
                        >
                            Authenticate
                        </button>
                    </div>
                </div>
                <div v-else class="auth-info">
                    <div class="alert alert-success">
                        <i class="bi bi-shield-check me-2"></i>
                        <strong>Authenticated</strong>
                    </div>
                    <InfoCard v-if="authService.claims.value">
                        <InfoCardRow label="Subject">
                            {{ authService.claims.value.sub || 'N/A' }}
                        </InfoCardRow>
                        <InfoCardRow label="Issuer">
                            {{ authService.claims.value.iss || 'N/A' }}
                        </InfoCardRow>
                        <InfoCardRow label="Audience">
                            {{ authService.claims.value.aud || 'N/A' }}
                        </InfoCardRow>
                        <InfoCardRow v-if="authService.claims.value.exp" label="Expires">
                            {{ tokenExpiryLabel }}
                        </InfoCardRow>
                        <div v-if="authService.tokenExpiryWarning.value" class="alert alert-warning mt-2">
                            <i class="bi bi-exclamation-triangle me-2"></i>
                            <span>Token expires soon!</span>
                        </div>
                    </InfoCard>
                    <div class="d-flex justify-content-end gap-2 mt-3">
                        <button type="button" class="btn btn-secondary" @click="emit('close')">Close</button>
                        <button type="button" class="btn btn-danger" @click="authService.logout()">
                            <i class="bi bi-box-arrow-right me-2"></i>
                            <span>Logout</span>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.auth-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.7);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
}

.auth-modal-content {
    background: var(--bg-primary);
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    max-height: 80vh;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.auth-modal-header {
    padding: 1.5rem;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.auth-modal-header h5 {
    margin: 0;
    color: var(--text-primary);
}

.auth-modal-body {
    padding: 1.5rem;
    overflow-y: auto;
}

.auth-form .form-label {
    color: var(--text-primary);
    margin-bottom: 0.5rem;
}

.auth-form textarea {
    font-family: 'Courier New', monospace;
    font-size: 0.9rem;
}

.auth-form .form-text {
    color: var(--text-muted);
}
</style>
