<script setup lang="ts">
import { nextTick, ref, useTemplateRef } from 'vue';
import { CONSTANTS } from '../constants';
import { sovd } from '../services/SovdService';
import { telemetryService } from '../services/TelemetryService';
import { SovdVersionInfo } from '../services/ApiService';

const serverUrl = globalThis.location.host || 'localhost';

const serverBanner = useTemplateRef('server-banner');
const versionPane = useTemplateRef('version-pane');

const isBannerHovered = ref(false);
const isPaneHovered = ref(false);
const showVersionPane = ref(false);

function showVersionPaneOnHover() {
    isBannerHovered.value = true;
    showVersionPane.value = true;
    nextTick(positionVersionPane);
}

function hideVersionPaneOnLeave() {
    isBannerHovered.value = false;
    scheduleVersionPaneHide();
}

function hidePaneOnLeave() {
    isPaneHovered.value = false;
    scheduleVersionPaneHide();
}

function scheduleVersionPaneHide() {
    setTimeout(() => {
        if (!isPaneHovered.value && !isBannerHovered.value) {
            showVersionPane.value = false;
        }
    }, CONSTANTS.PANE_HIDE_DELAY);
}

function keepVersionPaneOpen() {
    isPaneHovered.value = true;
}

function positionVersionPane() {
    try {
        if (!serverBanner.value || !versionPane.value) return;

        const rect = serverBanner.value.getBoundingClientRect();
        versionPane.value.style.top = `${rect.top}px`;
        versionPane.value.style.left = `${rect.right + 8}px`;
    } catch (error) {
        console.warn('Failed to position version pane:', error);
    }
}

function selectVersionFromPane(version: SovdVersionInfo) {
    showVersionPane.value = false;
    sovd.selectedVersion.value = version.base_uri;

    telemetryService.trackInteraction('change_version', {
        version: sovd.currentVersion.value?.version,
    });
    telemetryService.log('Version Changed', `Switched to API version ${sovd.currentVersion.value?.version}`);
    sovd.fetchComponents();
}

function formatVendorInfo(vendorInfo: unknown) {
    return vendorInfo && typeof vendorInfo === 'object' ? Object.entries(vendorInfo) : [];
}
</script>

<template>
    <div
        ref="server-banner"
        class="server-banner"
        @mouseenter="showVersionPaneOnHover"
        @mouseleave="hideVersionPaneOnLeave"
    >
        <div><i class="bi bi-hdd-network me-2"></i>{{ serverUrl }}</div>
        <div>
            <span v-if="sovd.currentVersion.value">{{ 'v' + sovd.currentVersion.value.version }}</span>
            <i class="bi bi-chevron-right ms-2"></i>
        </div>
    </div>

    <!-- Version Selection Pane -->
    <div
        ref="version-pane"
        class="version-pane"
        :class="{ open: showVersionPane }"
        @mouseenter="keepVersionPaneOpen"
        @mouseleave="hidePaneOnLeave"
    >
        <div class="version-pane-content">
            <div class="list-group list-group-flush">
                <div
                    v-for="(version, index) in sovd.versionInfo.value?.sovd_info ?? []"
                    :key="index"
                    class="list-group-item list-group-item-action"
                    :class="{ active: sovd.selectedVersion.value === version.base_uri }"
                    @click="selectVersionFromPane(version)"
                >
                    <div class="version-title">{{ 'API Version ' + version.version }}</div>
                    <div class="version-uri">{{ version.base_uri }}</div>
                    <div v-if="version.vendor_info" class="vendor-info">
                        <div
                            v-for="[key, value] in formatVendorInfo(version.vendor_info)"
                            :key="key"
                            class="vendor-info-item"
                        >
                            <span class="vendor-key">{{ key + ':' }}</span>
                            <span class="vendor-value">{{ value }}</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.server-banner {
    background-color: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: 4px;
    padding: 0.5rem;
    margin-bottom: 0.5rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.85rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    position: relative;
    cursor: pointer;
    transition:
        background-color 0.2s ease,
        color 0.2s ease;
}

.server-banner:hover {
    background-color: var(--bg-hover);
    color: var(--text-hover);
}

.server-banner .bi-chevron-right {
    font-size: 0.8rem;
}

.version-pane {
    position: fixed;
    top: auto;
    left: auto;
    width: 320px;
    max-height: none;
    background-color: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
    z-index: 1000;
    overflow: visible;
    opacity: 0;
    visibility: hidden;
    transform: translateY(-10px);
    transition: all 0.2s ease;
}

.version-pane.open {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
}

.version-pane-content {
    padding: 0;
}

.list-group-item {
    background-color: var(--bg-primary);
    border-color: var(--border-color);
    color: var(--text-primary);
}

.list-group-item:hover {
    background-color: rgba(255, 204, 0, 0.15);
    border-color: var(--accent-color);
    color: var(--text-primary);
}

.list-group-item.active,
.list-group-item.active.list-group-item-action,
.list-group-item.list-group-item-action.active,
.list-group-item-action.active {
    background-color: var(--bg-selected) !important;
    border-color: var(--accent-color) !important;
    color: #000000 !important;
}

.list-group-item.active:hover,
.list-group-item.active.list-group-item-action:hover,
.list-group-item.list-group-item-action.active:hover,
.list-group-item-action.active:hover {
    background-color: var(--bg-selected) !important;
    border-color: var(--accent-color) !important;
    color: #000000 !important;
}

.list-group-item-action.active,
.list-group-item-action.active:hover,
.list-group-item-action.active:focus {
    color: #000000 !important;
    background-color: var(--bg-selected) !important;
}

.list-group-item.active *,
.list-group-item-action.active * {
    color: #000000 !important;
}

.version-title {
    font-weight: 600;
    font-size: 0.9rem;
    margin-bottom: 0.25rem;
}

.version-uri {
    font-family: monospace;
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-bottom: 0.25rem;
}

.vendor-info {
    font-size: 0.75rem;
}

.vendor-info-item {
    display: flex;
    justify-content: space-between;
    margin: 0.15rem 0;
    padding: 0.1rem 0;
    border-bottom: 1px dotted var(--border-color);
}

.vendor-key {
    font-weight: 500;
    color: var(--text-secondary);
}

.vendor-value {
    color: var(--text-primary);
    text-align: right;
}
</style>
