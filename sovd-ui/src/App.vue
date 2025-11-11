<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, useTemplateRef, watch } from 'vue';
import Breadcrumbs from './components/Breadcrumbs.vue';
import ComponentDetails from './components/ComponentDetails.vue';
import ComponentTree from './components/ComponentTree.vue';
import KeyboardHelpModal from './components/KeyboardHelpModal.vue';
import ResourceDetails from './components/ResourceDetails.vue';
import ServerBanner from './components/ServerBanner.vue';
import TheHeader from './components/TheHeader.vue';
import Toast from './components/Toast.vue';
import { CONSTANTS } from './constants';
import { authService } from './services/AuthService';
import { notificationService } from './services/NotificationService';
import { NodePath, sovd } from './services/SovdService';
import { telemetryService } from './services/TelemetryService';

const selectedItem = ref<NodePath | null>(null);

const showKeyboardHelp = ref(false);

const telemetryInterval = ref<ReturnType<typeof setInterval> | null>(null);

// refetch components upon login/logout
watch(authService.token, () => {
    if (!sovd.versionInfo.value) return; // skip during app initialization
    sovd.fetchComponents();
});

watch(sovd.selectedVersion, () => {
    selectedItem.value = null;
});

const componentTree = useTemplateRef('component-tree');

const handleKeyDown = (event: KeyboardEvent) => {
    const key = event.key;
    const ctrlOrCmd = event.ctrlKey || event.metaKey;

    if (key === '/' || (ctrlOrCmd && key === 'k')) {
        event.preventDefault();
        componentTree.value?.focusFilterInput();
        return;
    }

    if (key === '?') {
        event.preventDefault();
        toggleKeyboardHelp();
        return;
    }

    if (key === 'Escape') {
        event.preventDefault();
        handleEscape();
    }
};

function toggleKeyboardHelp() {
    showKeyboardHelp.value = !showKeyboardHelp.value;
}

function handleEscape() {
    if (showKeyboardHelp.value) {
        showKeyboardHelp.value = false;
    } else if (selectedItem.value) {
        selectedItem.value = null;
    } else if (document.activeElement instanceof HTMLInputElement) {
        document.activeElement.blur();
    }
}

onMounted(() => {
    telemetryService.init();
    telemetryService.trackInteraction('page_load', {
        url: globalThis.location.href,
        referrer: document.referrer,
    });
    authService.init();
    sovd.fetchVersionInfo();
    sovd.fetchComponents();

    document.addEventListener('keydown', handleKeyDown);

    telemetryInterval.value = setInterval(() => {
        if (telemetryService.shouldLogSummary()) {
            const summary = telemetryService.getSummary();
        }
    }, CONSTANTS.TELEMETRY_LOG_INTERVAL);
});

onBeforeUnmount(() => {
    telemetryService.trackInteraction('page_unload', {
        sessionDuration: performance.now(),
    });

    document.removeEventListener('keydown', handleKeyDown);

    authService.deinit();

    if (telemetryInterval.value) {
        clearInterval(telemetryInterval.value);
    }

    telemetryService.cleanup();
});
</script>

<template>
    <TheHeader />

    <!-- Main content area -->
    <div class="content">
        <!-- Sidebar with component tree -->
        <div class="sidebar">
            <ServerBanner />
            <h6 class="mb-2 ms-2 mt-3 sidebar-title"><i class="bi bi-diagram-3 me-2"></i>Components</h6>

            <ComponentTree ref="component-tree" v-model="selectedItem" />
        </div>

        <!-- Main panel -->
        <div class="main-panel">
            <Breadcrumbs v-if="selectedItem" v-model="selectedItem" />

            <div v-if="!selectedItem" class="empty-state">
                <i class="bi bi-arrow-left-circle empty-state-icon"></i>
                <h5 class="mt-3">Select a component or resource</h5>
                <p>Choose a component from the left panel to view its details</p>
            </div>

            <ComponentDetails v-if="selectedItem && selectedItem.length === 1" :component-id="selectedItem[0]" />

            <ResourceDetails
                v-if="selectedItem && selectedItem.length === 2"
                :component-id="selectedItem[0]"
                :resource-key="selectedItem[1]"
            />
        </div>
    </div>

    <!-- Telemetry Toast Notifications -->
    <Toast
        v-if="notificationService.currentToast.value"
        :notification="notificationService.currentToast.value"
        @close="notificationService.currentToast.value = null"
    />

    <!-- Keyboard Shortcuts Help Modal -->
    <KeyboardHelpModal v-if="showKeyboardHelp" @close="showKeyboardHelp = false" />
</template>

<style scoped>
.content {
    flex: 1;
    display: flex;
    overflow: hidden;
}

.sidebar {
    width: 280px;
    background-color: var(--bg-sidebar);
    border-right: 1px solid var(--border-color);
    overflow-y: auto;
    padding: 0.75rem;
    position: relative;
}

.main-panel {
    flex: 1;
    padding: 1.5rem;
    overflow-y: auto;
    background-color: var(--bg-primary);
}

.sidebar-title {
    font-size: 0.95rem;
}

.empty-state-icon {
    font-size: 3rem;
}
</style>
