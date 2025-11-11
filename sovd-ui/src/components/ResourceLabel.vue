<script setup lang="ts">
import { computed } from 'vue';
import { formatToTitleCase } from '../util/string';
import ResourceIcon from './ResourceIcon.vue';

const props = defineProps<{
    resourceKey: string;
}>();

const label = computed(() => formatToTitleCase(props.resourceKey).replaceAll('-', ' '));
</script>

<template>
    <span>
        <ResourceIcon :resource-key="resourceKey" class="me-2" />
        <span>{{ label }}</span>
    </span>
</template>
