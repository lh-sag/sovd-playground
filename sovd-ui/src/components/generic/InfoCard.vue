<script setup lang="ts"></script>

<template>
    <div class="info-card">
        <slot />
    </div>
</template>

<style scoped>
.info-card {
    background-color: var(--bg-secondary);
    border-left: 3px solid var(--accent-color);
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 1rem;
}
</style>
