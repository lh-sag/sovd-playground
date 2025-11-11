<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { notificationService } from '../services/NotificationService';
import { sovd } from '../services/SovdService';
import { Level, telemetryService } from '../services/TelemetryService';
import InfoCard from './generic/InfoCard.vue';

const props = defineProps<{
    componentId: string;
    resourceKey: string;
}>();

const url = computed(() => sovd.componentResourceUrls.value.get(props.componentId)?.[props.resourceKey]);

// resource data
const resourceData = ref(null);
const isLoading = ref(false);
async function fetchData() {
    resourceData.value = null;
    isLoading.value = true;

    try {
        const response = await telemetryService.trackApiCall(url.value);

        if (!response.ok) {
            notificationService.showToast('Load Error', `Failed to load data (${response.status})`, Level.ERROR);
            return;
        }

        resourceData.value = await response.json();
    } catch (error) {
        telemetryService.logError('Network error', error);
    } finally {
        isLoading.value = false;
    }
}

// fetch data when URL changes
watch(url, () => fetchData(), { immediate: true });
</script>

<template>
    <div>
        <InfoCard v-if="resourceData">
            <h6 class="mb-3">Resource Data</h6>

            <!-- For data resources with items array -->
            <div v-if="resourceData.items && Array.isArray(resourceData.items)">
                <div v-if="resourceData.items.length === 0" class="alert alert-info">
                    <i class="bi bi-info-circle me-2"></i>
                    <span>No data items available in this resource.</span>
                </div>
                <div v-else>
                    <div class="table-responsive">
                        <table class="table table-sm data-table">
                            <thead>
                                <tr>
                                    <th>ID</th>
                                    <th>Name</th>
                                    <th>Category</th>
                                    <th>Groups</th>
                                    <th>Tags</th>
                                    <th>Read Only</th>
                                </tr>
                            </thead>
                            <tbody>
                                <tr v-for="item in resourceData.items" :key="item.id">
                                    <td class="data-table-cell">
                                        <code class="code-inline">{{ item.id }}</code>
                                    </td>
                                    <td class="data-table-cell">{{ item.name || 'N/A' }}</td>
                                    <td class="data-table-cell">
                                        <span class="badge badge-category">{{ item.category }}</span>
                                    </td>
                                    <td class="data-table-cell">
                                        <span
                                            v-for="group in item.groups || []"
                                            :key="group"
                                            class="badge badge-group me-1"
                                        >
                                            {{ group }}
                                        </span>
                                        <span v-if="!item.groups || item.groups.length === 0" class="text-none">
                                            None
                                        </span>
                                    </td>
                                    <td class="data-table-cell">
                                        <span v-for="tag in item.tags || []" :key="tag" class="badge badge-tag me-1">
                                            {{ tag }}
                                        </span>
                                        <span v-if="!item.tags || item.tags.length === 0" class="text-none">None</span>
                                    </td>
                                    <td class="data-table-cell">
                                        <i
                                            class="icon-status me-2"
                                            :class="item.read_only ? 'bi bi-lock locked' : 'bi bi-unlock unlocked'"
                                        ></i>
                                        <span>{{ item.read_only ? 'Yes' : 'No' }}</span>
                                    </td>
                                </tr>
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <!-- For other resource types, display as JSON -->
            <div v-else>
                <pre class="code-display"><code>{{ JSON.stringify(resourceData, null, 2) }}</code></pre>
            </div>
        </InfoCard>

        <div v-else-if="!isLoading" class="alert alert-info">
            <i class="bi bi-info-circle me-2"></i>
            <span>No data available for this resource.</span>
        </div>
    </div>
</template>

<style scoped>
.code-display {
    background-color: var(--bg-secondary);
    padding: 1rem;
    border-radius: 4px;
    color: var(--text-primary);
    font-size: 0.85rem;
    max-height: 400px;
    overflow-y: auto;
}

.data-table {
    --bs-table-bg: var(--bg-primary) !important;
    --bs-table-color: var(--text-primary) !important;
    --bs-table-border-color: var(--border-color) !important;
    color: var(--text-primary);
    background-color: var(--bg-primary);
    border-color: var(--border-color);
}

.data-table th {
    background-color: var(--bg-secondary) !important;
    color: var(--text-primary) !important;
    border-color: var(--border-color) !important;
}

.data-table-cell {
    background-color: var(--bg-primary) !important;
    color: var(--text-primary) !important;
    border-color: var(--border-color) !important;
}

.data-table tbody tr {
    background-color: var(--bg-primary) !important;
}

.icon-status.locked {
    color: var(--accent-color);
}

.icon-status.unlocked {
    color: #28a745;
}

.code-inline {
    font-size: 0.8rem;
    color: var(--text-primary);
    background-color: var(--bg-secondary);
    padding: 0.2rem 0.3rem;
    border-radius: 3px;
}

.badge-category {
    background-color: var(--accent-color);
    color: #000;
}

.badge-group {
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
}

.badge-tag {
    background-color: var(--text-secondary);
    color: var(--bg-primary);
}

.text-none {
    color: var(--text-muted);
}
</style>
