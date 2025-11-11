<script setup lang="ts">
import { computed, ref, useTemplateRef } from 'vue';
import { CONSTANTS } from '../constants';
import { NodePath, sovd } from '../services/SovdService';
import { telemetryService } from '../services/TelemetryService';
import ResourceLabel from './ResourceLabel.vue';
import TreeView from './generic/TreeView.vue';

const selectedItem = defineModel<NodePath | null>({ required: true });

const filterText = ref('');
const debouncedFilterText = ref('');

function debounce<F extends (...args: any) => any>(func: F, wait: number) {
    let timeout: ReturnType<typeof setTimeout>;
    return (...args: Parameters<F>) => {
        const later = () => {
            func(...args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

const updateDebouncedFilter = debounce((value: string) => {
    debouncedFilterText.value = value;
    telemetryService.trackInteraction('filter_components', {
        filter: value,
    });
}, CONSTANTS.SEARCH_DEBOUNCE);

const filteredComponents = computed(() => {
    if (!debouncedFilterText.value) {
        return sovd.components.value;
    }
    const searchTerm = debouncedFilterText.value.toLowerCase();
    return sovd.components.value.filter((component) => {
        const name = (component.name ?? component.id).toLowerCase();
        return name.includes(searchTerm);
    });
});

function onFilterInput() {
    updateDebouncedFilter(filterText.value);
}

const filterInput = useTemplateRef('filter-input');
function focusFilterInput() {
    filterInput.value?.focus();
    filterInput.value?.select();
}

const componentTree = useTemplateRef('component-tree');

function canExpand(path: NodePath): boolean {
    return path.length === 1;
}

function getChildren(path: NodePath): string[] {
    if (path.length === 0) {
        return filteredComponents.value.map((component) => component.id);
    } else if (path.length === 1) {
        const componentId = path[0];
        return Object.keys(sovd.componentResourceUrls.value.get(componentId) ?? {});
    }
    return [];
}

defineExpose({
    focusFilterInput,
});
</script>

<template>
    <div class="mb-2">
        <input
            ref="filter-input"
            type="text"
            v-model="filterText"
            @input="onFilterInput"
            @keydown.down="componentTree?.focusFirstNode()"
            class="form-control form-control-sm filter-input"
            placeholder="Filter components..."
            maxlength="100"
        />
    </div>

    <!-- Loading skeleton for components -->
    <div v-if="sovd.loadingComponents.value">
        <div v-for="i in 5" :key="'skeleton-' + i" class="skeleton skeleton-tree-item">
            <div class="skeleton skeleton-tree-icon"></div>
            <div class="skeleton skeleton-tree-text"></div>
        </div>
    </div>

    <div v-else-if="filteredComponents.length === 0" class="empty-state">
        <i class="bi bi-inbox"></i>
        <p class="mt-2">No components available</p>
    </div>

    <TreeView
        v-else
        ref="component-tree"
        v-model="selectedItem"
        :get-children
        :can-expand
        @focus-before="focusFilterInput"
        v-slot="{ path }"
    >
        <template v-if="path.length === 1">
            <i class="bi bi-box me-2"></i>
            <span>{{ sovd.components.value.find((c) => c.id === path[0])?.name ?? path[0] }}</span>
        </template>
        <template v-else-if="path.length === 2">
            <ResourceLabel :resource-key="path[1]" />
        </template>
    </TreeView>
</template>

<style scoped>
.filter-input {
    font-size: 0.85rem;
    background-color: var(--bg-primary);
    color: var(--text-primary);
    border-color: var(--border-color);
}

.filter-input:focus {
    border-color: var(--accent-color);
    box-shadow: 0 0 0 0.25rem rgba(255, 204, 0, 0.25);
}

.filter-input::placeholder {
    font-size: 0.85rem;
}
</style>
