<script setup lang="ts">
import { computed } from 'vue';
import { NodePath, sovd } from '../services/SovdService';
import { telemetryService } from '../services/TelemetryService';
import ResourceLabel from './ResourceLabel.vue';

const selectedItem = defineModel<NodePath | null>();

function clear() {
    telemetryService.trackInteraction('clear_selection');

    selectedItem.value = null;
}

const component = computed(() => {
    if (!selectedItem.value || selectedItem.value.length === 0) return undefined;
    const componentId = selectedItem.value[0];
    return sovd.components.value.find((c) => c.id === componentId);
});

function select(path: NodePath) {
    telemetryService.trackInteraction('breadcrumb_select');

    selectedItem.value = path;
}
</script>

<template>
    <nav aria-label="breadcrumb" class="mb-3 breadcrumb-wrapper">
        <ol class="breadcrumb">
            <li class="breadcrumb-item">
                <a href="#" @click.prevent="clear()" class="breadcrumb-link">
                    <i class="bi bi-diagram-3 me-2"></i>
                    <span>Components</span>
                </a>
            </li>
            <li
                v-if="selectedItem && selectedItem.length >= 1"
                class="breadcrumb-item"
                :class="{ active: selectedItem.length === 1 }"
            >
                <a href="#" @click.prevent="select(selectedItem.slice(0, 1))" class="breadcrumb-link">
                    <i class="bi bi-box me-2"></i>
                    <span>{{ component?.name ?? selectedItem[0] }}</span>
                </a>
            </li>
            <li v-if="selectedItem && selectedItem.length >= 2" class="breadcrumb-item active" aria-current="page">
                <ResourceLabel :resource-key="selectedItem[1]" />
            </li>
        </ol>
    </nav>
</template>

<style scoped>
.breadcrumb {
    background-color: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    padding: 0.5rem 0.75rem;
    margin: 0;
    font-size: 0.9rem;
}

.breadcrumb-item + .breadcrumb-item::before {
    content: '>';
    color: var(--text-muted);
}

.breadcrumb-link {
    color: var(--text-secondary);
    text-decoration: none;
}

.breadcrumb-link:hover {
    color: var(--text-primary) !important;
    text-decoration: underline !important;
}

.breadcrumb-item.active {
    font-weight: 500;
    color: var(--text-primary);
}
.breadcrumb-item.active .breadcrumb-link {
    color: var(--text-primary);
}

.breadcrumb-item i {
    font-size: 0.85rem;
}

.breadcrumb-wrapper {
    margin: -1.5rem -1.5rem 1rem -1.5rem;
}
</style>
