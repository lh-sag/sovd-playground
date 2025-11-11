<script setup lang="ts">
defineProps<{
    label: string;
}>();
</script>

<template>
    <div class="info-card-row">
        <div class="info-card-row__label">{{ label }}:</div>
        <div class="info-card-row__value">
            <slot />
        </div>
    </div>
</template>

<style scoped>
.info-card-row {
    display: flex;
    margin-bottom: 0.5rem;
}

.info-card-row__label {
    font-weight: 600;
    color: var(--text-secondary);
    min-width: 120px;
}

.info-card-row__value {
    color: var(--text-primary);
    word-break: break-all;
}
</style>
