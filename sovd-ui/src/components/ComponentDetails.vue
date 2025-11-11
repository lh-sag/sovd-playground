<script setup lang="ts">
import { ref, watch } from 'vue';
import { notificationService } from '../services/NotificationService';
import { sovd } from '../services/SovdService';
import { Level, telemetryService } from '../services/TelemetryService';
import { formatToTitleCase } from '../util/string';
import InfoCard from './generic/InfoCard.vue';
import InfoCardRow from './generic/InfoCardRow.vue';
import ResourceLabel from './ResourceLabel.vue';
import { ComponentResponse } from '../services/ApiService';

const props = defineProps<{
    componentId: string;
}>();

// fetch component details
const componentDetails = ref<ComponentResponse | null>(null);
const isLoading = ref(false);
async function fetchData() {
    componentDetails.value = null;

    isLoading.value = true;

    try {
        const response = await telemetryService.trackApiCall(
            `${sovd.apiBaseUrl.value}/components/${props.componentId}`,
        );

        if (!response.ok) {
            notificationService.showToast('Load Error', `Failed to load data (${response.status})`, Level.ERROR);
            return;
        }

        componentDetails.value = await response.json();
    } catch (error) {
        telemetryService.logError('Network error', error, { showToast: false });
    } finally {
        isLoading.value = false;
    }
}
watch(
    () => props.componentId,
    () => fetchData(),
    { immediate: true },
);

function formatExtensionKey(key: string) {
    return formatToTitleCase(key);
}

function formatExtensionValue(value: unknown) {
    if (value === null || value === undefined) {
        return 'N/A';
    }
    if (Array.isArray(value)) {
        return value.join(', ');
    }
    if (typeof value === 'object') {
        return JSON.stringify(value);
    }
    return value.toString();
}
</script>

<template>
    <div>
        <!-- Loading skeleton for details -->
        <div v-if="isLoading">
            <div class="skeleton skeleton-card"></div>
            <div class="skeleton skeleton-card"></div>
        </div>

        <div v-else-if="componentDetails">
            <InfoCard>
                <h6 class="mb-3">Component Information</h6>
                <InfoCardRow label="ID">
                    {{ componentDetails.id }}
                </InfoCardRow>
                <InfoCardRow label="Name">
                    {{ componentDetails.name || 'N/A' }}
                </InfoCardRow>
                <InfoCardRow v-if="componentDetails.translation_id" label="Translation ID">
                    {{ componentDetails.translation_id }}
                </InfoCardRow>

                <!-- Display component extensions if present -->
                <div v-if="componentDetails.extensions">
                    <hr class="my-2" />
                    <InfoCardRow
                        v-for="[key, value] in Object.entries(componentDetails.extensions)"
                        :key="key"
                        :label="formatExtensionKey(key)"
                    >
                        {{ formatExtensionValue(value) }}
                    </InfoCardRow>
                </div>
            </InfoCard>

            <InfoCard v-if="componentDetails.resources && Object.keys(componentDetails.resources).length > 0">
                <h6 class="mb-3">Available Resources</h6>
                <p class="text-muted">Expand the component in the tree view to access individual resources.</p>
                <ul class="list-unstyled mb-0">
                    <li
                        v-for="[key, _] in Object.entries(componentDetails.resources).filter(
                            ([key, _]) => key !== 'extensions',
                        )"
                        :key="key"
                    >
                        <ResourceLabel :resource-key="key" />
                    </li>
                </ul>

                <!-- Display resource extensions if present -->
                <div v-if="componentDetails.resources && componentDetails.resources.extensions" class="mt-2">
                    <strong>Resource Extensions:</strong>
                    <div
                        v-for="[key, value] in Object.entries(componentDetails.resources.extensions)"
                        :key="key"
                        class="ms-3 resource-extensions"
                    >
                        {{ formatExtensionKey(key) + ': ' + formatExtensionValue(value) }}
                    </div>
                </div>
            </InfoCard>

            <div v-else class="alert alert-info">
                <i class="bi bi-info-circle me-2"></i>
                <span>No resources available for this component</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.resource-extensions {
    font-size: 0.9rem;
}
</style>
