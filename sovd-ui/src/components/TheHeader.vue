<script setup lang="ts">
import { ref, watch } from 'vue';
import { authService } from '../services/AuthService';
import { notificationService } from '../services/NotificationService';
import { Level, telemetryService } from '../services/TelemetryService';
import AuthenticationModal from './AuthenticationModal.vue';

const showAuthModal = ref(false);

watch(telemetryService.authenticationFailureDetected, (authFailure) => {
    if (authFailure) {
        showAuthModal.value = true;
        telemetryService.authenticationFailureDetected.value = false;
    }
});

watch(authService.tokenExpiryWarning, (tokenExpiryWarning) => {
    if (tokenExpiryWarning && !showAuthModal.value) {
        notificationService.showToast(
            'Token Expiring Soon',
            'Your token will expire in less than 5 minutes',
            Level.WARNING,
        );
    }
});
</script>

<template>
    <nav class="topbar">
        <div class="container-fluid d-flex justify-content-between align-items-center">
            <div class="header-title">SOVD</div>
            <div class="auth-status">
                <button
                    v-if="authService.token.value"
                    class="btn btn-sm btn-outline-light"
                    @click="showAuthModal = true"
                    :title="'Authenticated as ' + (authService.claims.value?.sub || 'User')"
                >
                    <i class="bi bi-shield-check me-2"></i>
                    <span>{{ authService.claims.value?.sub || 'Authenticated' }}</span>
                </button>
                <button v-else class="btn btn-sm btn-outline-light" @click="showAuthModal = true">
                    <i class="bi bi-shield-x me-2"></i>
                    <span>Sign In</span>
                </button>
            </div>
        </div>
    </nav>

    <!-- Authentication Modal -->
    <AuthenticationModal v-if="showAuthModal" @close="showAuthModal = false" />
</template>

<style scoped>
.topbar {
    background-color: var(--bg-topbar);
    color: var(--text-primary);
    padding: 0.5rem;
    border-bottom: 1px solid var(--border-color);
}

.header-title {
    font-weight: bold;
    color: rgba(255, 255, 255, 0.9);
}

.auth-status {
    display: flex;
    align-items: center;
}

.auth-status .btn-outline-light {
    border-color: rgba(255, 255, 255, 0.3);
    color: rgba(255, 255, 255, 0.9);
}

.auth-status .btn-outline-light:hover {
    background-color: rgba(255, 255, 255, 0.1);
    border-color: rgba(255, 255, 255, 0.5);
}
</style>
