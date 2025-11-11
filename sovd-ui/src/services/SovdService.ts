import { computed, ref } from 'vue';
import { api, Component, VersionInfoResponse } from './ApiService';
import { telemetryService } from './TelemetryService';

export type NodePath = string[];

export class SovdService {
    versionInfo = ref<VersionInfoResponse | null>(null);
    selectedVersion = ref<string | null>(null);

    currentVersion = computed(() => {
        const sovdInfo = this.versionInfo.value?.sovd_info;
        if (!sovdInfo) return null;

        if (this.selectedVersion.value) {
            return sovdInfo.find((v) => v.base_uri === this.selectedVersion.value);
        }
        return sovdInfo[0];
    });

    apiBaseUrl = computed(() => {
        const version = this.currentVersion.value;
        return version ? version.base_uri : '/sovd/v1';
    });

    async fetchVersionInfo() {
        try {
            const data = await api.getVersionInfo();
            if (data.sovd_info.length > 0) {
                this.versionInfo.value = data;
                this.selectedVersion.value = data.sovd_info[0].base_uri;
                telemetryService.log('Version Info', `Loaded ${data.sovd_info.length} API versions`);
            }
        } catch (error) {
            telemetryService.logError('Network error', error);
        }
    }

    components = ref<Component[]>([]);
    componentResourceUrls = ref<Map<string, Record<string, string>>>(new Map());
    loadingComponents = ref(false);

    async fetchComponents() {
        this.loadingComponents.value = true;

        try {
            const data = await api.getComponents(this.apiBaseUrl.value);
            this.components.value = data.items ?? [];

            await Promise.allSettled(
                this.components.value.map((component) => this.fetchComponentResources(component.id)),
            );

            telemetryService.log('Components Loaded', `Found ${this.components.value.length} components`);
        } catch (error) {
            telemetryService.logError('Network error', error);
            this.components.value = [];
        } finally {
            this.loadingComponents.value = false;
        }
    }

    async fetchComponentResources(componentId: string) {
        try {
            const resources = await api.getComponentResources(this.apiBaseUrl.value, componentId);
            this.componentResourceUrls.value.set(componentId, resources);
        } catch (error) {
            telemetryService.logError(`Failed to fetch resources for ${componentId}`, error, {
                showToast: false,
                consoleLevel: 'error',
            });
        }
    }
}

export const sovd = new SovdService();
