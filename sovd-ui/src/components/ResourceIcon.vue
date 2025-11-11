<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
    resourceKey: string;
}>();

const IconMap: Record<string, string> = {
    configurations: 'bi bi-gear',
    faults: 'bi bi-exclamation-triangle',
    operations: 'bi bi-play-circle',
    updates: 'bi bi-arrow-clockwise',
    modes: 'bi bi-toggles',
    locks: 'bi bi-lock',
    logs: 'bi bi-file-text',
    communication_logs: 'bi bi-chat-left-text',
    scripts: 'bi bi-code-slash',
    triggers: 'bi bi-lightning',
    bulk_data: 'bi bi-archive',
    cyclic_subscriptions: 'bi bi-arrow-repeat',
};

const classNames = computed(() => {
    // Handle data variants
    if (props.resourceKey.includes('data')) {
        return 'bi bi-database';
    }

    return IconMap[props.resourceKey] || 'bi bi-circle';
});
</script>

<template>
    <i :class="classNames" />
</template>
