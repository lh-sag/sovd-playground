import { telemetryService } from './TelemetryService';

export class ApiService {
    async getVersionInfo(): Promise<VersionInfoResponse> {
        const response = await telemetryService.trackApiCall('/sovd/version-info');
        if (!response.ok) throw new Error(`API Error: ${response.status} ${response.statusText}`);
        return await response.json();
    }

    async getComponents(baseUrl: string): Promise<ComponentsResponse> {
        const response = await telemetryService.trackApiCall(`${baseUrl}/components`);
        if (!response.ok) throw new Error(`API Error: ${response.status} ${response.statusText}`);
        return await response.json();
    }

    async getComponentResources(baseUrl: string, componentId: string): Promise<Record<string, string>> {
        const response = await telemetryService.trackApiCall(`${baseUrl}/components/${componentId}`);
        if (!response.ok) throw new Error(`API Error: ${response.status} ${response.statusText}`);

        const data = await response.json();

        const result: Record<string, string> = {};
        for (const [key, value] of Object.entries(data)) {
            if (
                value &&
                typeof value === 'string' &&
                key !== 'id' &&
                key !== 'name' &&
                key !== 'extensions' &&
                key !== 'resources'
            ) {
                result[key] = value;
            }
        }
        for (const [key, value] of Object.entries(data.resources ?? {})) {
            if (value && typeof value === 'string' && key !== 'extensions' && !(key in result)) {
                result[key] = value;
            }
        }

        return result;
    }
}

export interface VersionInfoResponse {
    sovd_info: SovdVersionInfo[];
}

export interface SovdVersionInfo {
    version: string;
    base_uri: string;
    vendor_info: Record<string, unknown>;
}

export interface ComponentsResponse {
    items: Component[];
}

export interface Component {
    id: string;
    name: string;
    href: string;
}

export interface ComponentResources {
    id: string;
    name: string;
}

export interface ComponentResponse {
    id: string;
    name: string;
    [key: string]: any;
}

export const api = new ApiService();
