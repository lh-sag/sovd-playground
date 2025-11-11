<script setup lang="ts">
import { onMounted, ref } from 'vue';
import type { Notification } from '../services/NotificationService';
import { Level } from '../services/TelemetryService';

defineProps<{ notification: Notification }>();

const emit = defineEmits<{
    close: [];
}>();

// delayed visibility for transition
const isMounted = ref(false);
onMounted(async () => {
    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            isMounted.value = true;
        });
    });
});
</script>

<template>
    <div class="telemetry-toast" :class="{ [notification.level]: true, show: isMounted }">
        <div class="d-flex align-items-center">
            <i v-if="notification.level === Level.ERROR" class="bi bi-exclamation-circle-fill me-2"></i>
            <i v-else-if="notification.level === Level.WARNING" class="bi bi-exclamation-triangle-fill me-2"></i>
            <i v-else class="bi bi-info-circle-fill me-2"></i>
            <div class="flex-grow-1">
                <strong>{{ notification.title }}</strong>
                <div>{{ notification.message }}</div>
            </div>
            <button type="button" class="btn-close btn-close-sm ms-2" @click="emit('close')"></button>
        </div>
    </div>
</template>

<style scoped>
.telemetry-toast {
    position: fixed;
    bottom: 20px;
    right: 20px;
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 0.75rem 1rem;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
    z-index: 1050;
    max-width: 350px;
    opacity: 0;
    transform: translateY(20px);
    transition: all 0.3s ease;
}

.telemetry-toast.show {
    opacity: 1;
    transform: translateY(0);
}

.telemetry-toast.error {
    border-left: 3px solid #dc3545;
}

.telemetry-toast.warning {
    border-left: 3px solid #ffc107;
}

.telemetry-toast.info {
    border-left: 3px solid var(--accent-color);
}
</style>
