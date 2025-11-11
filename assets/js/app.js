// SPDX-FileCopyrightText: Copyright Liebherr-Digital Development Center GmbH
// SPDX-License-Identifier: Apache-2.0

const AuthService = {
    TOKEN_KEY: 'sovd_auth_token',
    TOKEN_EXPIRY_WARNING: 5 * 60 * 1000,
    TOKEN_CHECK_INTERVAL: 5 * 60 * 1000,

    parseJWT(token) {
        try {
            if (!token || typeof token !== 'string') {
                throw new Error('Invalid token type');
            }
            const parts = token.split('.');
            if (parts.length !== 3) {
                throw new Error('Invalid JWT format');
            }
            const payload = parts[1];
            const decoded = atob(payload.replace(/-/g, '+').replace(/_/g, '/'));
            return JSON.parse(decoded);
        } catch (e) {
            console.error('Failed to parse JWT:', e);
            return null;
        }
    },

    saveToken(token, persist = false) {
        const storage = persist ? localStorage : sessionStorage;
        storage.setItem(this.TOKEN_KEY, token);
        if (persist) {
            localStorage.setItem(`${this.TOKEN_KEY}_persist`, 'true');
        } else {
            localStorage.removeItem(`${this.TOKEN_KEY}_persist`);
        }
    },

    loadToken() {
        const usePersistent = localStorage.getItem(`${this.TOKEN_KEY}_persist`) === 'true';
        const storage = usePersistent ? localStorage : sessionStorage;
        return storage.getItem(this.TOKEN_KEY);
    },

    clearToken() {
        sessionStorage.removeItem(this.TOKEN_KEY);
        localStorage.removeItem(this.TOKEN_KEY);
        localStorage.removeItem(`${this.TOKEN_KEY}_persist`);
    },

    getSecondsUntilExpiry(claims) {
        if (!claims?.exp) {
            return Infinity; // No expiry
        }
        const now = Math.floor(Date.now() / 1000);
        return claims.exp - now;
    },

    isTokenExpired(claims) {
        return this.getSecondsUntilExpiry(claims) < 0;
    },

    isTokenExpiringSoon(claims) {
        const seconds = this.getSecondsUntilExpiry(claims);
        return seconds > 0 && seconds <= (this.TOKEN_EXPIRY_WARNING / 1000);
    },

    getAuthHeader(token) {
        return token ? { 'Authorization': `Bearer ${token}` } : {};
    }
};

const TelemetryService = {
    metrics: {
        apiCalls: [],
        errors: [],
        interactions: [],
        performance: [],
    },
    lastMetricsHash: null,

    levels: {
        INFO: "info",
        WARNING: "warning",
        ERROR: "error",
    },

    init() {

        window.addEventListener("error", (event) => {
            this.logError("Global Error", {
                message: event.message,
                source: event.filename,
                line: event.lineno,
                column: event.colno,
                error: event.error,
            });
        });

        window.addEventListener("unhandledrejection", (event) => {
            this.logError("Unhandled Promise Rejection", {
                reason: event.reason,
                promise: event.promise,
            });
        });

    },

    async trackApiCall(url, method = "GET", options = {}) {
        const {
            headers = {},
            body = null,
            authToken = null,
            fetchOptions = {}
        } = options;

        const startTime = performance.now();

        // Build default headers
        const defaultHeaders = {
            "Content-Type": "application/json",
            ...AuthService.getAuthHeader(authToken),
            ...headers
        };

        // Build fetch configuration
        const fetchConfig = {
            method,
            headers: defaultHeaders,
            ...fetchOptions
        };

        if (body && method !== 'GET') {
            fetchConfig.body = typeof body === 'string' ? body : JSON.stringify(body);
        }

        const metric = {
            url,
            method,
            timestamp: new Date().toISOString(),
            duration: null,
            status: null,
            error: null,
            authenticated: !!authToken,
        };

        try {
            const response = await fetch(url, fetchConfig);
            metric.duration = performance.now() - startTime;
            metric.status = response?.status ?? "unknown";

            this.addMetric('apiCalls', metric);

            if (metric.duration > CONSTANTS.SLOW_API_THRESHOLD) {
                this.log(
                    "Slow API Call",
                    `${method} ${url} took ${metric.duration.toFixed(0)}ms`,
                    this.levels.WARNING,
                );
            }

            if (response.status === 401) {
                this.log(
                    "Authentication Required",
                    `${method} ${url} requires authentication`,
                    this.levels.WARNING,
                );

                if (appInstance.value && appInstance.value.handleAuthenticationRequired) {
                    appInstance.value.handleAuthenticationRequired();
                }
            }

            return response;
        } catch (error) {
            metric.duration = performance.now() - startTime;
            metric.error = error.message;
            metric.status = "error";

            this.addMetric('apiCalls', metric);
            this.logError(
                "API Call Failed",
                { url, method, error: error.message },
                { showToast: false }
            );

            throw error;
        }
    },


    trackInteraction(action, details = {}) {
        const interaction = {
            action,
            details,
            timestamp: new Date().toISOString(),
            viewport: {
                width: window.innerWidth,
                height: window.innerHeight,
            },
            userAgent: navigator.userAgent,
        };

        this.addMetric('interactions', interaction);
    },

    logError(context, details = {}, options = {}) {
        const {
            consoleLevel = 'error',  // 'error', 'warn', 'log', or false
            showToast = true,
            toastMessage = null
        } = options;

        const error = {
            context,
            details,
            timestamp: new Date().toISOString(),
            stack: new Error().stack,
        };

        this.addMetric('errors', error);

        // Flexible console output
        if (consoleLevel) {
            const message = details.error || details.message || details;
            const consoleMsg = typeof details === 'object'
                ? `[Telemetry] ${context}:`
                : `[Telemetry] ${context}: ${details}`;

            console[consoleLevel](consoleMsg, typeof details === 'object' ? details : '');
        }

        // Show toast with custom or default message
        if (showToast) {
            const message = toastMessage || "An error occurred. Check console for details.";
            this.showToast(context, message, this.levels.ERROR);
        }
    },

    log(title, message, level = this.levels.INFO) {
        const logEntry = {
            title,
            message,
            level,
            timestamp: new Date().toISOString(),
        };

        switch (level) {
            case this.levels.ERROR:
                console.error(`[Telemetry] ${title}:`, message);
                break;
            case this.levels.WARNING:
                console.warn(`[Telemetry] ${title}:`, message);
                break;
            default:
                console.log(`[Telemetry] ${title}:`, message);
        }
    },

    showToast(title, message, level = this.levels.INFO) {
        const toast = document.getElementById("telemetry-toast");
        if (!toast) return;

        const titleEl = toast.querySelector(".toast-title");
        const messageEl = toast.querySelector(".toast-message");
        const iconEl = toast.querySelector(".bi");

        if (!titleEl || !messageEl || !iconEl) return;

        titleEl.textContent = title;
        messageEl.textContent = message;
        toast.className = `telemetry-toast show ${level}`;

        const iconClasses = {
            [this.levels.ERROR]: "bi bi-exclamation-circle-fill me-2",
            [this.levels.WARNING]: "bi bi-exclamation-triangle-fill me-2",
        };

        iconEl.className = iconClasses[level] ?? "bi bi-info-circle-fill me-2";

        setTimeout(() => {
            toast?.classList.remove("show");
        }, CONSTANTS.TOAST_TIMEOUT);
    },

    getSummary() {
        const apiCalls = this.metrics.apiCalls;
        const totalDuration = apiCalls.reduce((sum, c) => sum + (c.duration || 0), 0);

        return {
            apiCalls: {
                total: apiCalls.length,
                failed: apiCalls.filter(c => c.error).length,
                averageDuration: apiCalls.length ? totalDuration / apiCalls.length : 0,
            },
            errors: this.metrics.errors.length,
            interactions: this.metrics.interactions.length,
            uptime: performance.now(),
        };
    },

    addMetric(type, metric) {
        const metrics = this.metrics[type];
        if (metrics.length >= CONSTANTS.MAX_METRICS_SIZE) {
            const retentionSize = Math.floor(CONSTANTS.MAX_METRICS_SIZE * 0.8);
            this.metrics[type] = metrics.slice(-retentionSize);
        }
        this.metrics[type].push(metric);
    },

    getMetricsHash() {
        const summary = this.getSummary();
        return JSON.stringify(summary);
    },

    shouldLogSummary() {
        const currentHash = this.getMetricsHash();
        if (currentHash !== this.lastMetricsHash) {
            this.lastMetricsHash = currentHash;
            return true;
        }
        return false;
    },

    cleanup() {
        this.metrics = {
            apiCalls: [],
            errors: [],
            interactions: [],
            performance: [],
        };
        this.lastMetricsHash = null;
    },

    exportMetrics() {
        return JSON.stringify(this.metrics, null, 2);
    },
};

function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const context = this;
        const later = () => {
            func.apply(context, args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

// Simplified API call helper to reduce duplication
async function apiCall(url, options = {}) {
    const {
        method = 'GET',
        body = null,
        authToken = null,
        showToast = true,
        telemetry = null
    } = options;

    try {
        const response = await (telemetry || TelemetryService).trackApiCall(
            url,
            method,
            { authToken, body }
        );

        if (!response.ok) {
            if (appInstance.value) {
                appInstance.value.handleApiError(response, url, showToast);
            }
            return { ok: false, data: null, response };
        }

        const data = await response.json();
        return { ok: true, data, response };
    } catch (error) {
        if (telemetry) {
            telemetry.logError("Network error", error);
        }
        return { ok: false, data: null, error };
    }
}

const { createApp } = Vue;

const appInstance = { value: null };

const CONSTANTS = {
    MAX_METRICS_SIZE: 1000,
    SLOW_API_THRESHOLD: 1000,
    TOAST_TIMEOUT: 5000,
    SEARCH_DEBOUNCE: 300,
    PANE_HIDE_DELAY: 150,
    TELEMETRY_LOG_INTERVAL: 30000
};

const appConfig = {
    data() {
        return {
            versionInfo: null,
            serverUrl: window.location.host || "localhost",
            selectedVersion: null,
            showVersionPane: false,
            versionPaneHoverTimeout: null,
            components: [],
            componentsWithResources: [],
            selectedItem: null,
            componentDetails: null,
            resourceData: null,
            loadingComponents: false,
            loadingDetails: false,
            expandedComponents: {},
            filterText: "",
            debouncedFilterText: "",
            telemetry: TelemetryService,

            focusedIndex: -1,
            focusedResourceIndex: -1,
            showKeyboardHelp: false,
            keyboardMode: false,

            authToken: null,
            authTokenClaims: null,
            authTokenExpiry: null,

            // Data value management
            expandedDataItem: null,
            dataValues: new Map(),
            loadingDataValue: false,
            dataValueError: null,
            editingDataItem: null,
            editingDataValue: '',
            dataValueValidationError: null,
            savingDataValue: false,
            authTokenInput: "",
            persistToken: false,
            showAuthModal: false,
            tokenExpiryWarning: false,
            tokenExpiryCheckInterval: null,

            resourceCache: new Map(),
            telemetryInterval: null,
        };
    },
    computed: {
        filteredComponents() {
            if (!this.debouncedFilterText) {
                return this.componentsWithResources;
            }
            const searchTerm = this.debouncedFilterText.toLowerCase();
            return this.componentsWithResources.filter((component) => {
                const name = (component.name ?? component.id).toLowerCase();
                return name.includes(searchTerm);
            });
        },

        currentVersion() {
            const sovdInfo = this.versionInfo?.sovd_info;
            if (!sovdInfo) return null;

            if (this.selectedVersion) {
                return sovdInfo.find(v => v.base_uri === this.selectedVersion);
            }
            return sovdInfo[0];
        },

        apiBaseUrl() {
            const version = this.currentVersion;
            return version ? version.base_uri : "/sovd/v1";
        },
    },
    methods: {

        updateDebouncedFilter: debounce(function (value) {
            this.debouncedFilterText = value;
            this.telemetry.trackInteraction("filter_components", {
                filter: value,
            });
        }, CONSTANTS.SEARCH_DEBOUNCE),

        onFilterInput(event) {
            this.updateDebouncedFilter(this.filterText);
        },

        handleApiError(response, url, showToast = true) {
            if (response.status === 401) {
                this.handleAuthenticationRequired();
                return null;
            }

            this.telemetry.logError(
                `API Error: ${response.status} ${response.statusText}`,
                { url, status: response.status },
                { showToast: false }
            );

            if (showToast) {
                this.telemetry.showToast(
                    "Load Error",
                    `Failed to load data (${response.status})`,
                    this.telemetry.levels.ERROR
                );
            }
            return null;
        },

        getCachedData(key, ttlMs = 300000) {
            const cached = this.resourceCache.get(key);
            if (cached && (Date.now() - cached.timestamp) < ttlMs) {
                return cached.data;
            }
            return null;
        },

        setCachedData(key, data) {
            this.resourceCache.set(key, {
                data: data,
                timestamp: Date.now()
            });
        },

        clearCache() {
            this.resourceCache.clear();
        },

        async fetchVersionInfo() {
            const result = await apiCall("/sovd/version-info", {
                authToken: this.authToken,
                showToast: false,
                telemetry: this.telemetry
            });

            if (result.ok && result.data?.sovd_info?.length > 0) {
                this.versionInfo = result.data;
                this.selectedVersion = result.data.sovd_info[0].base_uri;
                this.telemetry.log(
                    "Version Info",
                    `Loaded ${result.data.sovd_info.length} API versions`,
                );
            }
        },

        async fetchComponents() {
            this.loadingComponents = true;

            const result = await apiCall(`${this.apiBaseUrl}/components`, {
                authToken: this.authToken,
                showToast: false,
                telemetry: this.telemetry
            });

            if (result.ok) {
                this.components = result.data.items ?? [];
                await this.fetchAllComponentResources();
                this.telemetry.log(
                    "Components Loaded",
                    `Found ${this.components.length} components`
                );
            } else {
                this.components = [];
                this.componentsWithResources = [];
            }

            this.loadingComponents = false;
        },

        async fetchAllComponentResources() {
            // Track which components are subcomponents
            const subcomponentIds = new Set();

            const fetchComponentResource = async (component, depth = 0) => {
                try {
                    const response = await this.telemetry.trackApiCall(
                        `${this.apiBaseUrl}/components/${component.id}`,
                        "GET",
                        { authToken: this.authToken }
                    );

                    if (!response.ok) {
                        return component;
                    }

                    const capabilities = await response.json();
                    const resources = this.extractResources(capabilities);

                    // Initialize the enhanced component
                    const enhancedComponent = {
                        ...component,
                        resources,
                        subcomponents: [],
                        hasSubcomponents: false
                    };

                    // Check if this component has subcomponents
                    if (capabilities.subcomponents) {
                        enhancedComponent.hasSubcomponents = true;

                        // Only fetch subcomponents for first level to avoid deep nesting on initial load
                        if (depth === 0) {
                            const subcomponentsData = await this.fetchSubcomponents(component.id);
                            if (subcomponentsData && subcomponentsData.length > 0) {
                                // Track subcomponent IDs so we can filter them out from top level
                                subcomponentsData.forEach(sub => subcomponentIds.add(sub.id));

                                // Fetch resources for each subcomponent
                                const subcomponentPromises = subcomponentsData.map(sub =>
                                    fetchComponentResource(sub, depth + 1)
                                );
                                const subResults = await Promise.allSettled(subcomponentPromises);
                                enhancedComponent.subcomponents = subResults
                                    .filter(result => result.status === 'fulfilled')
                                    .map(result => result.value);
                            }
                        }
                    }

                    return enhancedComponent;
                } catch (error) {
                    this.telemetry.logError(
                        `Failed to fetch resources for ${component.id}`,
                        error,
                        { showToast: false, consoleLevel: 'error' }
                    );
                    return component;
                }
            };

            const results = await Promise.allSettled(
                this.components.map(comp => fetchComponentResource(comp, 0))
            );

            // Filter out subcomponents from the top level list
            this.componentsWithResources = results
                .filter(result => result.status === 'fulfilled')
                .map(result => result.value)
                .filter(comp => !subcomponentIds.has(comp.id));
        },

        async fetchSubcomponents(componentId) {
            try {
                const response = await this.telemetry.trackApiCall(
                    `${this.apiBaseUrl}/components/${componentId}/subcomponents`,
                    "GET",
                    { authToken: this.authToken }
                );

                if (!response.ok) {
                    return [];
                }

                const data = await response.json();
                return data.items || [];
            } catch (error) {
                this.telemetry.logError(
                    `Failed to fetch subcomponents for ${componentId}`,
                    error,
                    { showToast: false, consoleLevel: 'warn' }
                );
                return [];
            }
        },

        extractResources(capabilities) {
            const excludedKeys = new Set(['id', 'name', 'extensions']);

            const directResources = Object.entries(capabilities)
                .filter(([key, value]) =>
                    value &&
                    typeof value === "string" &&
                    !excludedKeys.has(key)
                )
                .map(([key, value]) => ({ key, value }));

            if (!capabilities.resources) {
                return directResources;
            }

            const additionalResources = Object.entries(capabilities.resources)
                .filter(([key, value]) =>
                    value &&
                    key !== 'extensions' &&
                    !directResources.some(r => r.key === key)
                )
                .map(([key, value]) => ({ key, value }));

            return [...directResources, ...additionalResources];
        },

        toggleExpand(componentId) {
            // Use Vue 3's reactivity properly by creating a new object
            this.expandedComponents = {
                ...this.expandedComponents,
                [componentId]: !this.expandedComponents[componentId]
            };
        },

        async fetchComponentDetails(componentId, options = {}) {
            const { showToast = true } = options;

            try {
                const response = await this.telemetry.trackApiCall(
                    `${this.apiBaseUrl}/components/${componentId}`,
                    "GET",
                    { authToken: this.authToken }
                );

                if (!response.ok) {
                    this.handleApiError(response, `${this.apiBaseUrl}/components/${componentId}`, showToast);
                    return null;
                }

                return await response.json();
            } catch (error) {
                this.telemetry.logError(
                    "Network error",
                    error,
                    { showToast }
                );
                return null;
            }
        },

        resetSelectionState() {
            this.componentDetails = null;
            this.resourceData = null;
        },

        setComponentSelection(id, name) {
            this.selectedItem = {
                type: "component",
                id: id,
                name: name
            };
            this.resetSelectionState();
            this.loadingDetails = true;
        },

        async selectComponent(component) {
            this.telemetry.trackInteraction("select_component", {
                id: component.id,
            });

            // Ensure the component is expanded (don't toggle, just set to true)
            this.expandedComponents = {
                ...this.expandedComponents,
                [component.id]: true
            };

            this.setComponentSelection(component.id, component.name);

            try {
                this.componentDetails = await this.fetchComponentDetails(component.id);
            } finally {
                this.loadingDetails = false;
            }
        },

        async selectResource(componentOrEvent, resource) {
            // Handle both old format (component, resource) and new event format ({ component, resource })
            let component, res;
            if (resource) {
                // Old format: direct parameters
                component = componentOrEvent;
                res = resource;
            } else {
                // New format: event object
                component = componentOrEvent.component;
                res = componentOrEvent.resource;
            }

            this.telemetry.trackInteraction("select_resource", {
                componentId: component.id,
                resource: res.key,
            });

            this.selectedItem = {
                type: "resource",
                key: res.key,
                value: res.value,
                componentId: component.id,
                componentName: component.name ?? component.id,
            };
            this.componentDetails = null;
            this.resourceData = null;
            this.loadingDetails = true;

            try {
                const response = await this.telemetry.trackApiCall(
                    res.value,
                    "GET",
                    { authToken: this.authToken }
                );

                if (!response.ok) {
                    this.handleApiError(response, res.value, true);
                    this.resourceData = null;
                    return;
                }

                this.resourceData = await response.json();
            } catch (error) {
                this.telemetry.logError("Network error", error);
                this.resourceData = null;
            } finally {
                this.loadingDetails = false;
            }
        },

        formatToTitleCase(str, separator = '_') {
            return str
                .replace(/([A-Z])/g, " $1")
                .replace(new RegExp(separator, 'g'), " ")
                .split(" ")
                .filter(word => word)
                .map(word => word.charAt(0).toUpperCase() + word.slice(1))
                .join(" ")
                .trim();
        },

        formatResourceName(name) {
            return this.formatToTitleCase(name).replace(/-/g, " ");
        },

        getResourceIcon(resourceKey) {
            const iconMap = {
                configurations: "bi bi-gear",
                faults: "bi bi-exclamation-triangle",
                operations: "bi bi-play-circle",
                updates: "bi bi-arrow-clockwise",
                modes: "bi bi-toggles",
                locks: "bi bi-lock",
                logs: "bi bi-file-text",
                communication_logs: "bi bi-chat-left-text",
                scripts: "bi bi-code-slash",
                triggers: "bi bi-lightning",
                bulk_data: "bi bi-archive",
                cyclic_subscriptions: "bi bi-arrow-repeat",
            };

            // Handle data variants
            if (resourceKey.includes('data')) {
                return "bi bi-database";
            }

            return iconMap[resourceKey] || "bi bi-circle";
        },

        formatExtensionKey(key) {
            return this.formatToTitleCase(key);
        },

        formatExtensionValue(value) {
            if (value === null || value === undefined) {
                return "N/A";
            }
            if (Array.isArray(value)) {
                return value.join(", ");
            }
            if (typeof value === "object") {
                return JSON.stringify(value);
            }
            return value.toString();
        },

        // Data value management methods
        async toggleDataItem(item) {
            if (this.expandedDataItem === item.id) {
                // Collapse if already expanded
                this.expandedDataItem = null;
                this.editingDataItem = null;
                this.dataValueError = null;
            } else {
                // Expand and fetch value
                this.expandedDataItem = item.id;
                this.editingDataItem = null;
                this.dataValueError = null;

                // Fetch value if not already cached
                if (!this.dataValues.has(item.id)) {
                    await this.fetchDataValue(item);
                }
            }
        },

        async fetchDataValue(item) {
            this.loadingDataValue = true;
            this.dataValueError = null;

            try {
                const componentId = this.selectedItem.componentId;
                const url = `${this.selectedVersion}/components/${componentId}/data/${item.id}?include-schema=true`;

                const response = await this.telemetry.trackApiCall(
                    url,
                    "GET",
                    { authToken: this.authToken }
                );

                if (!response.ok) {
                    const error = await response.text();
                    this.dataValueError = `Failed to load data value: ${response.status}`;
                    return;
                }

                const result = await response.json();
                this.dataValues.set(item.id, result.data);
            } catch (error) {
                this.dataValueError = `Error loading data value: ${error.message}`;
                this.telemetry.logError("Failed to fetch data value", error);
            } finally {
                this.loadingDataValue = false;
            }
        },

        async refreshDataValue(item) {
            this.dataValues.delete(item.id);
            await this.fetchDataValue(item);
        },

        enterEditMode(item) {
            this.editingDataItem = item.id;
            const currentValue = this.dataValues.get(item.id);
            this.editingDataValue = this.formatJson(currentValue);
            this.dataValueValidationError = null;
        },

        cancelEdit(item) {
            this.editingDataItem = null;
            this.editingDataValue = '';
            this.dataValueValidationError = null;
        },

        validateDataValue() {
            try {
                JSON.parse(this.editingDataValue);
                this.dataValueValidationError = null;
            } catch (e) {
                this.dataValueValidationError = `Invalid JSON: ${e.message}`;
            }
        },

        async saveDataValue(item) {
            this.validateDataValue();
            if (this.dataValueValidationError) {
                return;
            }

            this.savingDataValue = true;

            try {
                const componentId = this.selectedItem.componentId;
                const url = `${this.selectedVersion}/components/${componentId}/data/${item.id}`;
                const newValue = JSON.parse(this.editingDataValue);

                const response = await this.telemetry.trackApiCall(
                    url,
                    "PUT",
                    {
                        authToken: this.authToken,
                        body: { data: newValue }
                    }
                );

                if (!response.ok) {
                    const error = await response.text();
                    this.telemetry.showToast(
                        "Save Failed",
                        `Failed to save data value: ${response.status}`,
                        this.telemetry.levels.ERROR
                    );
                    return;
                }

                // Update cached value
                this.dataValues.set(item.id, newValue);
                this.editingDataItem = null;
                this.editingDataValue = '';

                this.telemetry.showToast(
                    "Success",
                    "Data value saved successfully",
                    this.telemetry.levels.INFO
                );
            } catch (error) {
                this.telemetry.logError("Failed to save data value", error);
                this.telemetry.showToast(
                    "Save Error",
                    `Error saving data value: ${error.message}`,
                    this.telemetry.levels.ERROR
                );
            } finally {
                this.savingDataValue = false;
            }
        },

        async copyDataValue(item) {
            const value = this.dataValues.get(item.id);
            const text = this.formatJson(value);

            try {
                await navigator.clipboard.writeText(text);
                this.telemetry.showToast(
                    "Copied",
                    "Data value copied to clipboard",
                    this.telemetry.levels.INFO
                );
            } catch (error) {
                this.telemetry.logError("Failed to copy to clipboard", error);
            }
        },

        formatJson(value) {
            if (value === null || value === undefined) {
                return 'null';
            }
            return JSON.stringify(value, null, 2);
        },

        async onVersionChange() {
            this.telemetry.trackInteraction("change_version", {
                version: this.currentVersion?.version,
            });
            this.telemetry.log(
                "Version Changed",
                `Switched to API version ${this.currentVersion?.version}`,
            );
            this.selectedItem = null;
            this.componentDetails = null;
            this.clearCache();
            await this.fetchComponents();
        },

        // Simplified version pane management
        setVersionPaneVisibility(visible, delayed = false) {
            // Clear any pending timeout
            if (this.versionPaneHoverTimeout) {
                clearTimeout(this.versionPaneHoverTimeout);
                this.versionPaneHoverTimeout = null;
            }

            if (!visible && delayed) {
                // Schedule hide with delay
                this.versionPaneHoverTimeout = setTimeout(() => {
                    this.showVersionPane = false;
                }, CONSTANTS.PANE_HIDE_DELAY);
            } else {
                // Immediate show/hide
                this.showVersionPane = visible;
                if (visible) {
                    this.$nextTick(() => this.positionVersionPane());
                }
            }
        },

        positionVersionPane() {
            try {
                const serverBanner = document.querySelector(".server-banner");
                const versionPane = document.querySelector(".version-pane");

                if (!serverBanner || !versionPane) return;

                const rect = serverBanner.getBoundingClientRect();
                versionPane.style.top = `${rect.top}px`;
                versionPane.style.left = `${rect.right + 8}px`;
            } catch (error) {
                console.warn('Failed to position version pane:', error);
            }
        },

        selectVersionFromPane(version) {
            this.selectedVersion = version.base_uri;
            this.showVersionPane = false;
            this.onVersionChange();
        },

        formatVendorInfo(vendorInfo) {
            return vendorInfo && typeof vendorInfo === "object"
                ? Object.entries(vendorInfo)
                : [];
        },

        clearSelection() {
            this.telemetry.trackInteraction("clear_selection");
            this.selectedItem = null;
            this.resetSelectionState();
        },

        async selectComponentFromBreadcrumb() {
            if (!this.selectedItem) return;

            this.telemetry.trackInteraction("breadcrumb_navigation", {
                target: "component",
            });

            let componentId;
            let componentName;

            if (this.selectedItem.type === "component") {
                componentId = this.selectedItem.id;
                componentName = this.selectedItem.name;
            } else if (this.selectedItem.type === "resource") {
                componentId = this.selectedItem.componentId;
                componentName = this.selectedItem.componentName;
            }

            if (!componentId) return;

            const component = this.componentsWithResources.find(
                (c) => c.id === componentId,
            );

            if (!component) return;

            this.expandedComponents[componentId] = true;
            this.setComponentSelection(componentId, componentName);

            try {
                this.componentDetails = await this.fetchComponentDetails(componentId, { showToast: false });
            } finally {
                this.loadingDetails = false;
            }
        },

        hideTelemetryToast() {
            const toast = document.getElementById("telemetry-toast");
            toast?.classList.remove("show");
        },

        handleKeyDown(event) {
            const key = event.key;
            const ctrlOrCmd = event.ctrlKey || event.metaKey;

            if (key === '/' || (ctrlOrCmd && key === 'k')) {
                event.preventDefault();
                this.focusFilterInput();
                return;
            }

            if (key === '?') {
                event.preventDefault();
                this.toggleKeyboardHelp();
                return;
            }

            if (key === 'Escape') {
                event.preventDefault();
                this.handleEscape();
                return;
            }

            if (this.keyboardMode && this.filteredComponents.length > 0) {
                switch(key) {
                    case 'ArrowDown':
                        event.preventDefault();
                        this.navigateDown();
                        break;
                    case 'ArrowUp':
                        event.preventDefault();
                        this.navigateUp();
                        break;
                    case 'ArrowRight':
                        event.preventDefault();
                        this.expandFocused();
                        break;
                    case 'ArrowLeft':
                        event.preventDefault();
                        this.collapseFocused();
                        break;
                    case 'Enter':
                        event.preventDefault();
                        this.selectFocused();
                        break;
                    case 'Home':
                        event.preventDefault();
                        this.focusFirst();
                        break;
                    case 'End':
                        event.preventDefault();
                        this.focusLast();
                        break;
                    case 'Tab':
                        this.keyboardMode = true;
                        break;
                }
            }
        },

        focusFilterInput() {
            try {
                const filterInput = document.querySelector('.filter-input');
                if (!filterInput) return;

                filterInput.focus();
                filterInput.select();
                this.keyboardMode = true;
            } catch (error) {
                console.warn('Failed to focus filter input:', error);
            }
        },

        toggleKeyboardHelp() {
            this.showKeyboardHelp = !this.showKeyboardHelp;
        },

        handleEscape() {
            if (this.showKeyboardHelp) {
                this.showKeyboardHelp = false;
            } else if (this.selectedItem) {
                this.clearSelection();
            } else {
                document.activeElement?.blur();
                this.resetFocusIndices();
            }
        },

        getNextNavigationPosition() {
            // Initial focus
            if (this.focusedIndex === -1) {
                return { componentIndex: 0, resourceIndex: -1 };
            }

            const currentComponent = this.filteredComponents[this.focusedIndex];

            // Move into resources if expanded
            if (this.hasExpandedResources(currentComponent) && this.focusedResourceIndex === -1) {
                return { componentIndex: this.focusedIndex, resourceIndex: 0 };
            }

            // Move to next resource
            if (this.focusedResourceIndex !== -1 &&
                this.focusedResourceIndex < currentComponent.resources.length - 1) {
                return { componentIndex: this.focusedIndex, resourceIndex: this.focusedResourceIndex + 1 };
            }

            // Move to next component
            if (this.focusedIndex < this.filteredComponents.length - 1) {
                return { componentIndex: this.focusedIndex + 1, resourceIndex: -1 };
            }

            // Stay at current position
            return { componentIndex: this.focusedIndex, resourceIndex: this.focusedResourceIndex };
        },

        getPreviousNavigationPosition() {
            // Move to previous resource
            if (this.focusedResourceIndex > 0) {
                return { componentIndex: this.focusedIndex, resourceIndex: this.focusedResourceIndex - 1 };
            }

            // Move from first resource to component
            if (this.focusedResourceIndex === 0) {
                return { componentIndex: this.focusedIndex, resourceIndex: -1 };
            }

            // Move to previous component
            if (this.focusedIndex > 0) {
                const prevIndex = this.focusedIndex - 1;
                const prevComponent = this.filteredComponents[prevIndex];
                const resourceIndex = this.hasExpandedResources(prevComponent)
                    ? prevComponent.resources.length - 1
                    : -1;
                return { componentIndex: prevIndex, resourceIndex };
            }

            // Stay at current position
            return { componentIndex: this.focusedIndex, resourceIndex: this.focusedResourceIndex };
        },

        navigateDown() {
            const { componentIndex, resourceIndex } = this.getNextNavigationPosition();
            this.focusedIndex = componentIndex;
            this.focusedResourceIndex = resourceIndex;
            this.scrollToFocused();
        },

        navigateUp() {
            const { componentIndex, resourceIndex } = this.getPreviousNavigationPosition();
            this.focusedIndex = componentIndex;
            this.focusedResourceIndex = resourceIndex;
            this.scrollToFocused();
        },

        expandFocused() {
            if (this.focusedIndex >= 0 && this.focusedResourceIndex === -1) {
                const component = this.filteredComponents[this.focusedIndex];
                if (component.resources?.length > 0) {
                    this.expandedComponents[component.id] = true;
                }
            }
        },

        collapseFocused() {
            if (this.focusedIndex >= 0) {
                const component = this.filteredComponents[this.focusedIndex];
                this.expandedComponents[component.id] = false;
                this.focusedResourceIndex = -1;
            }
        },

        selectFocused() {
            if (this.focusedIndex >= 0) {
                const component = this.filteredComponents[this.focusedIndex];
                if (this.focusedResourceIndex >= 0) {

                    const resource = component.resources[this.focusedResourceIndex];
                    this.selectResource(component, resource);
                } else {

                    this.selectComponent(component);
                }
            }
        },

        resetFocusIndices() {
            this.focusedIndex = -1;
            this.focusedResourceIndex = -1;
        },

        hasExpandedResources(component) {
            return this.expandedComponents[component.id] &&
                   component.resources?.length > 0;
        },

        focusFirst() {
            this.focusedIndex = 0;
            this.focusedResourceIndex = -1;
            this.scrollToFocused();
        },

        focusLast() {
            this.focusedIndex = this.filteredComponents.length - 1;
            const lastComponent = this.filteredComponents[this.focusedIndex];

            if (this.hasExpandedResources(lastComponent)) {
                this.focusedResourceIndex = lastComponent.resources.length - 1;
            } else {
                this.focusedResourceIndex = -1;
            }
            this.scrollToFocused();
        },

        scrollToFocused() {
            this.$nextTick(() => {
                if (this.focusedIndex < 0) return;

                const component = this.filteredComponents[this.focusedIndex];
                const selector = this.focusedResourceIndex >= 0
                    ? `[data-resource-id="${component.id}-${this.focusedResourceIndex}"]`
                    : `[data-component-id="${component.id}"]`;

                const focusedElement = document.querySelector(selector);
                focusedElement?.scrollIntoView({
                    behavior: 'smooth',
                    block: 'nearest'
                });
            });
        },

        isFocused(componentIndex, resourceIndex = -1) {
            return this.keyboardMode &&
                   this.focusedIndex === componentIndex &&
                   this.focusedResourceIndex === resourceIndex;
        },

        submitAuthToken() {
            const token = this.authTokenInput.trim();
            if (!token) return;

            const claims = AuthService.parseJWT(token);
            if (!claims) {
                this.telemetry.showToast(
                    "Invalid Token",
                    "The provided token is not a valid JWT",
                    this.telemetry.levels.ERROR
                );
                return;
            }

            if (AuthService.isTokenExpired(claims)) {
                this.telemetry.showToast(
                    "Token Expired",
                    "The provided token has already expired",
                    this.telemetry.levels.ERROR
                );
                return;
            }

            AuthService.saveToken(token, this.persistToken);

            this.setAuthToken(token, claims);
            this.authTokenInput = "";
            this.startTokenExpiryCheck();
            this.telemetry.log(
                "Authentication Success",
                `Authenticated as ${claims.sub || 'User'}`
            );
            this.fetchComponents();
        },

        setAuthToken(token, claims) {
            this.authToken = token;
            this.authTokenClaims = claims;
            this.authTokenExpiry = claims?.exp;
        },

        clearAuthToken() {
            this.authToken = null;
            this.authTokenClaims = null;
            this.authTokenExpiry = null;
            this.authTokenInput = "";
            this.persistToken = false;
            this.tokenExpiryWarning = false;
        },

        logout() {
            AuthService.clearToken();
            this.clearAuthToken();
            this.stopTokenExpiryCheck();

            this.telemetry.log("Logout", "User logged out");
            this.fetchComponents();
        },

        handleAuthenticationRequired() {
            this.showAuthModal = true;
            if (this.authToken) {
                this.telemetry.showToast(
                    "Authentication Failed",
                    "Your token may have expired. Please re-authenticate.",
                    this.telemetry.levels.WARNING
                );
            }
        },

        stopTokenExpiryCheck() {
            if (this.tokenExpiryCheckInterval) {
                clearInterval(this.tokenExpiryCheckInterval);
                this.tokenExpiryCheckInterval = null;
            }
        },

        startTokenExpiryCheck() {
            this.stopTokenExpiryCheck();
            this.tokenExpiryCheckInterval = setInterval(() => {
                if (this.authTokenClaims) {
                    if (AuthService.isTokenExpired(this.authTokenClaims)) {
                        this.telemetry.showToast(
                            "Token Expired",
                            "Your authentication token has expired. Please sign in again.",
                            this.telemetry.levels.WARNING
                        );
                        this.logout();
                    } else if (AuthService.isTokenExpiringSoon(this.authTokenClaims)) {
                        this.tokenExpiryWarning = true;
                        if (!this.showAuthModal) {
                            this.telemetry.showToast(
                                "Token Expiring Soon",
                                "Your token will expire in less than 5 minutes",
                                this.telemetry.levels.WARNING
                            );
                        }
                    } else {
                        this.tokenExpiryWarning = false;
                    }
                }
            }, AuthService.TOKEN_CHECK_INTERVAL);
        },

        formatTokenExpiry() {
            if (!this.authTokenExpiry) return 'Unknown';

            try {
                const diff = this.authTokenExpiry - Math.floor(Date.now() / 1000);

                if (diff < 0) return 'Expired';
                if (diff < 60) return `${diff}s`;
                if (diff < 3600) return `${Math.floor(diff / 60)}m`;
                if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
                return `${Math.floor(diff / 86400)}d`;
            } catch (error) {
                console.warn('Failed to format token expiry:', error);
                return 'Unknown';
            }
        },

        loadAuthToken() {
            const token = AuthService.loadToken();
            if (token) {
                const claims = AuthService.parseJWT(token);
                if (claims && !AuthService.isTokenExpired(claims)) {
                    this.setAuthToken(token, claims);
                    this.persistToken = localStorage.getItem(`${AuthService.TOKEN_KEY}_persist`) === 'true';
                    this.startTokenExpiryCheck();
                } else {

                    AuthService.clearToken();
                }
            }
        },
    },
    mounted() {
        this.telemetry.init();
        this.telemetry.trackInteraction("page_load", {
            url: window.location.href,
            referrer: document.referrer,
        });
        this.loadAuthToken();
        this.fetchVersionInfo();
        this.fetchComponents();
        appInstance.value = this;
        this.handleKeyDown = this.handleKeyDown.bind(this);
        this.handleMouseDown = () => {
            this.keyboardMode = false;
            this.resetFocusIndices();
        };

        document.addEventListener('keydown', this.handleKeyDown);
        document.addEventListener('mousedown', this.handleMouseDown);

        this.telemetryInterval = setInterval(() => {
            if (this.telemetry.shouldLogSummary()) {
                const summary = this.telemetry.getSummary();
            }
        }, CONSTANTS.TELEMETRY_LOG_INTERVAL);
    },
    beforeUnmount() {
        this.telemetry.trackInteraction("page_unload", {
            sessionDuration: performance.now(),
        });

        document.removeEventListener('keydown', this.handleKeyDown);
        document.removeEventListener('mousedown', this.handleMouseDown);

        if (this.tokenExpiryCheckInterval) {
            clearInterval(this.tokenExpiryCheckInterval);
        }

        if (this.telemetryInterval) {
            clearInterval(this.telemetryInterval);
        }

        this.telemetry.cleanup();
        this.clearCache();
        appInstance.value = null;

    },
};

const app = createApp(appConfig);

// Register the recursive component-tree-item component
app.component('component-tree-item', {
    template: '#component-tree-item-template',
    props: {
        component: {
            type: Object,
            required: true
        },
        level: {
            type: Number,
            default: 0
        },
        selectedItem: {
            type: Object,
            default: null
        },
        expandedComponents: {
            type: Object,
            required: true
        },
        getResourceIcon: {
            type: Function,
            required: true
        },
        formatResourceName: {
            type: Function,
            required: true
        }
    },
    computed: {
        isExpanded() {
            return this.expandedComponents[this.component.id] || false;
        },
        hasExpandableContent() {
            return (this.component.resources && this.component.resources.length > 0) ||
                   (this.component.subcomponents && this.component.subcomponents.length > 0);
        }
    },
    methods: {
        handleComponentClick() {
            // Don't emit toggle-expand here since selectComponent already handles it
            this.$emit('select-component', this.component);
        },
        handleResourceClick(resource) {
            this.$emit('select-resource', {
                component: this.component,
                resource: resource
            });
        }
    }
});

app.config.errorHandler = (err, vm, info) => {
    console.error("Vue Error:", err, info);
};

app.mount("#app");
