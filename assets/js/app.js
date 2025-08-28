/* OpenSOVD Web Interface - Application JavaScript */

// Authentication Service
const AuthService = {
    TOKEN_KEY: 'sovd_auth_token',
    TOKEN_EXPIRY_WARNING: 5 * 60 * 1000, // 5 minutes in milliseconds
    TOKEN_CHECK_INTERVAL: 5 * 60 * 1000, // Check every 5 minutes
    
    // Parse JWT without verification (for client-side display only)
    parseJWT(token) {
        try {
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
    
    // Save token to storage
    saveToken(token, persist = false) {
        const storage = persist ? localStorage : sessionStorage;
        storage.setItem(this.TOKEN_KEY, token);
        if (persist) {
            localStorage.setItem(this.TOKEN_KEY + '_persist', 'true');
        } else {
            localStorage.removeItem(this.TOKEN_KEY + '_persist');
        }
    },
    
    // Load token from storage
    loadToken() {
        // Check if we should use persistent storage
        const usePersistent = localStorage.getItem(this.TOKEN_KEY + '_persist') === 'true';
        const storage = usePersistent ? localStorage : sessionStorage;
        return storage.getItem(this.TOKEN_KEY);
    },
    
    // Clear token from storage
    clearToken() {
        sessionStorage.removeItem(this.TOKEN_KEY);
        localStorage.removeItem(this.TOKEN_KEY);
        localStorage.removeItem(this.TOKEN_KEY + '_persist');
    },
    
    // Check if token is expired
    isTokenExpired(claims) {
        if (!claims || !claims.exp) {
            return false; // Can't determine, assume valid
        }
        const now = Math.floor(Date.now() / 1000);
        return now >= claims.exp;
    },
    
    // Check if token expires soon
    isTokenExpiringSoon(claims) {
        if (!claims || !claims.exp) {
            return false;
        }
        const now = Math.floor(Date.now() / 1000);
        const expiryTime = claims.exp * 1000; // Convert to milliseconds
        const timeUntilExpiry = expiryTime - Date.now();
        return timeUntilExpiry > 0 && timeUntilExpiry <= this.TOKEN_EXPIRY_WARNING;
    },
    
    // Get authorization header
    getAuthHeader(token) {
        return token ? { 'Authorization': `Bearer ${token}` } : {};
    }
};

// Telemetry Service
const TelemetryService = {
    metrics: {
        apiCalls: [],
        errors: [],
        interactions: [],
        performance: [],
    },

    // Log levels
    levels: {
        INFO: "info",
        WARNING: "warning",
        ERROR: "error",
    },

    // Initialize telemetry
    init() {
        // Setup global error handler
        window.addEventListener("error", (event) => {
            this.logError("Global Error", {
                message: event.message,
                source: event.filename,
                line: event.lineno,
                column: event.colno,
                error: event.error,
            });
        });

        // Setup unhandled promise rejection handler
        window.addEventListener("unhandledrejection", (event) => {
            this.logError("Unhandled Promise Rejection", {
                reason: event.reason,
                promise: event.promise,
            });
        });

        console.log("[Telemetry] Service initialized");
    },

    // Log API call metrics (modified to support auth)
    async trackApiCall(url, method = "GET", fetchPromise, authToken = null) {
        const startTime = performance.now();
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
            const response = await fetchPromise;
            metric.duration = performance.now() - startTime;
            metric.status = response?.status || "unknown";

            this.metrics.apiCalls.push(metric);

            if (metric.duration > 1000) {
                this.log(
                    "Slow API Call",
                    `${method} ${url} took ${metric.duration.toFixed(0)}ms`,
                    this.levels.WARNING,
                );
            }

            // Handle 401 Unauthorized responses
            if (response.status === 401) {
                this.log(
                    "Authentication Required",
                    `${method} ${url} requires authentication`,
                    this.levels.WARNING,
                );
                // Trigger re-authentication in Vue app
                if (window.vueApp) {
                    window.vueApp.handleAuthenticationRequired();
                }
            }

            return response;
        } catch (error) {
            metric.duration = performance.now() - startTime;
            metric.error = error.message;
            metric.status = "error";

            this.metrics.apiCalls.push(metric);
            this.logError("API Call Failed", {
                url,
                method,
                error: error.message,
            });

            throw error;
        }
    },

    // Log user interactions
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

        this.metrics.interactions.push(interaction);
        console.log("[Telemetry] Interaction:", action, details);
    },

    // Log errors
    logError(context, details = {}) {
        const error = {
            context,
            details,
            timestamp: new Date().toISOString(),
            stack: new Error().stack,
        };

        this.metrics.errors.push(error);
        console.error("[Telemetry] Error:", context, details);

        // Show toast notification for errors
        this.showToast(
            context,
            "An error occurred. Check console for details.",
            this.levels.ERROR,
        );
    },

    // General logging
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

    // Show toast notification
    showToast(title, message, level = this.levels.INFO) {
        const toast = document.getElementById("telemetry-toast");
        if (!toast) return;

        const titleEl = toast.querySelector(".toast-title");
        const messageEl = toast.querySelector(".toast-message");
        const iconEl = toast.querySelector(".bi");

        titleEl.textContent = title;
        messageEl.textContent = message;

        // Set icon and style based on level
        toast.className = "telemetry-toast show " + level;

        switch (level) {
            case this.levels.ERROR:
                iconEl.className = "bi bi-exclamation-circle-fill me-2";
                break;
            case this.levels.WARNING:
                iconEl.className = "bi bi-exclamation-triangle-fill me-2";
                break;
            default:
                iconEl.className = "bi bi-info-circle-fill me-2";
        }

        // Auto-hide after 5 seconds
        setTimeout(() => {
            toast.classList.remove("show");
        }, 5000);
    },

    // Get telemetry summary
    getSummary() {
        return {
            apiCalls: {
                total: this.metrics.apiCalls.length,
                failed: this.metrics.apiCalls.filter((c) => c.error).length,
                averageDuration:
                    this.metrics.apiCalls.reduce(
                        (sum, c) => sum + (c.duration || 0),
                        0,
                    ) / (this.metrics.apiCalls.length || 1),
            },
            errors: this.metrics.errors.length,
            interactions: this.metrics.interactions.length,
            uptime: performance.now(),
        };
    },

    // Export metrics (for debugging or external services)
    exportMetrics() {
        return JSON.stringify(this.metrics, null, 2);
    },
};

// Debounce utility function
function debounce(func, wait) {
    let timeout;
    return function executedFunction(...args) {
        const context = this;
        const later = () => {
            clearTimeout(timeout);
            func.apply(context, args);
        };
        clearTimeout(timeout);
        timeout = setTimeout(later, wait);
    };
}

// Vue.js Application
const { createApp } = Vue;

// Create the Vue app
const appConfig = {
    data() {
        return {
            appName: "OpenSOVD",
            versionInfo: null,
            serverUrl: window.location.host || "localhost",
            selectedVersion: null,
            showVersionPane: false,
            isPaneHovered: false,
            isBannerHovered: false,
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
            // Keyboard navigation state
            focusedIndex: -1,
            focusedResourceIndex: -1,
            showKeyboardHelp: false,
            keyboardMode: false, // Track if user is using keyboard navigation
            // Authentication state
            authToken: null,
            authTokenClaims: null,
            authTokenExpiry: null,
            authTokenInput: "",
            persistToken: false,
            showAuthModal: false,
            tokenExpiryWarning: false,
            tokenExpiryCheckInterval: null,
        };
    },
    computed: {
        filteredComponents() {
            if (!this.debouncedFilterText) {
                return this.componentsWithResources;
            }
            const searchTerm = this.debouncedFilterText.toLowerCase();
            return this.componentsWithResources.filter((component) => {
                const name = (component.name || component.id).toLowerCase();
                return name.includes(searchTerm);
            });
        },

        currentVersion() {
            if (this.versionInfo && this.versionInfo.sovd_info) {
                if (this.selectedVersion) {
                    return this.versionInfo.sovd_info.find(
                        (v) => v.base_uri === this.selectedVersion,
                    );
                }
                return this.versionInfo.sovd_info[0];
            }
            return null;
        },

        apiBaseUrl() {
            const version = this.currentVersion;
            return version ? version.base_uri : "/opensovd/v1";
        },
    },
    methods: {
        // Debounced filter update
        updateDebouncedFilter: debounce(function (value) {
            this.debouncedFilterText = value;
            this.telemetry.trackInteraction("filter_components", {
                filter: value,
            });
        }, 300),

        onFilterInput(event) {
            this.updateDebouncedFilter(this.filterText);
        },

        async fetchVersionInfo() {
            const fetchPromise = fetch("/opensovd/version-info", {
                headers: {
                    "Content-Type": "application/json",
                    ...AuthService.getAuthHeader(this.authToken),
                },
            });

            await this.telemetry
                .trackApiCall("/opensovd/version-info", "GET", fetchPromise, this.authToken)
                .then((response) => {
                    if (response.ok) {
                        return response.json();
                    }
                    // Handle 401 specially - trigger auth modal
                    if (response.status === 401) {
                        this.handleAuthenticationRequired();
                        return null;
                    }
                    // Log other errors but don't throw
                    console.error(`API Error: ${response.status} ${response.statusText}`);
                    this.telemetry.logError("API Error", {
                        url: "/opensovd/version-info",
                        status: response.status,
                        statusText: response.statusText
                    });
                    return null;
                })
                .then((data) => {
                    if (data && data.sovd_info && data.sovd_info.length > 0) {
                        this.versionInfo = data;
                        // Set initial selected version
                        if (data.sovd_info && data.sovd_info.length > 0) {
                            this.selectedVersion = data.sovd_info[0].base_uri;
                        }
                        this.telemetry.log(
                            "Version Info",
                            `Loaded ${data.sovd_info.length} API versions`,
                        );
                    }
                })
                .catch((error) => {
                    // Network errors (not HTTP errors) still handled here
                    this.telemetry.logError(
                        "Failed to fetch version info",
                        error,
                    );
                    console.error("Network error fetching version info:", error);
                });
        },

        async fetchComponents() {
            this.loadingComponents = true;
            const fetchPromise = fetch(`${this.apiBaseUrl}/components`, {
                headers: {
                    "Content-Type": "application/json",
                    ...AuthService.getAuthHeader(this.authToken),
                },
            });

            await this.telemetry
                .trackApiCall(
                    `${this.apiBaseUrl}/components`,
                    "GET",
                    fetchPromise,
                    this.authToken
                )
                .then((response) => {
                    if (response.ok) {
                        return response.json();
                    }
                    // Handle 401 specially - trigger auth modal
                    if (response.status === 401) {
                        this.handleAuthenticationRequired();
                        return null;
                    }
                    // Log other errors but don't throw
                    console.error(`API Error: ${response.status} ${response.statusText}`);
                    this.telemetry.logError("API Error", {
                        url: `${this.apiBaseUrl}/components`,
                        status: response.status,
                        statusText: response.statusText
                    });
                    return null;
                })
                .then(async (data) => {
                    if (data) {
                        this.components = data.items || [];
                        await this.fetchAllComponentResources();
                        this.telemetry.log(
                            "Components Loaded",
                            `Found ${this.components.length} components`,
                        );
                    } else {
                        this.components = [];
                        this.componentsWithResources = [];
                    }
                })
                .catch((error) => {
                    // Network errors (not HTTP errors) still handled here
                    this.telemetry.logError(
                        "Failed to fetch components",
                        error,
                    );
                    this.components = [];
                    this.componentsWithResources = [];
                    console.error("Network error fetching components:", error);
                })
                .finally(() => {
                    this.loadingComponents = false;
                });
        },

        async fetchAllComponentResources() {
            // Use Promise.all for parallel fetching
            const promises = this.components.map((component) => {
                const fetchPromise = fetch(
                    `${this.apiBaseUrl}/components/${component.id}`,
                    {
                        headers: {
                            "Content-Type": "application/json",
                            ...AuthService.getAuthHeader(this.authToken),
                        },
                    },
                );

                return this.telemetry
                    .trackApiCall(
                        `${this.apiBaseUrl}/components/${component.id}`,
                        "GET",
                        fetchPromise,
                        this.authToken
                    )
                    .then((response) => {
                        if (response.ok) {
                            return response.json();
                        }
                        // Handle 401 specially - but don't show modal for each component
                        if (response.status === 401) {
                            // Auth will be handled by main fetchComponents
                            return null;
                        }
                        // Silently handle other errors for individual components
                        return null;
                    })
                    .then((capabilities) => {
                        if (capabilities) {
                            const resources =
                                this.extractResources(capabilities);
                            return {
                                ...component,
                                resources: resources,
                            };
                        } else {
                            return component;
                        }
                    })
                    .catch((error) => {
                        // Network errors - return component without resources
                        console.error(`Failed to fetch resources for ${component.id}:`, error);
                        return component;
                    });
            });

            this.componentsWithResources = await Promise.all(promises);
        },

        extractResources(capabilities) {
            const resources = [];
            // Extract all properties that look like resources
            for (const [key, value] of Object.entries(capabilities)) {
                if (
                    value &&
                    typeof value === "string" &&
                    key !== "id" &&
                    key !== "name" &&
                    key !== "extensions"
                ) {
                    resources.push({
                        key,
                        value: value,
                    });
                }
            }

            // Also check resources object if it exists
            if (capabilities.resources) {
                for (const [key, value] of Object.entries(
                    capabilities.resources,
                )) {
                    if (
                        value &&
                        key !== "extensions" &&
                        !resources.find((r) => r.key === key)
                    ) {
                        resources.push({ key, value });
                    }
                }
            }

            return resources;
        },

        toggleExpand(componentId) {
            this.expandedComponents[componentId] =
                !this.expandedComponents[componentId];
        },

        async selectComponent(component) {
            this.telemetry.trackInteraction("select_component", {
                id: component.id,
            });

            // Toggle expansion to show/hide resources
            this.toggleExpand(component.id);

            this.selectedItem = {
                type: "component",
                id: component.id,
                name: component.name,
            };
            this.componentDetails = null;
            this.resourceData = null;
            this.loadingDetails = true;

            const fetchPromise = fetch(
                `${this.apiBaseUrl}/components/${component.id}`,
                {
                    headers: {
                        "Content-Type": "application/json",
                        ...AuthService.getAuthHeader(this.authToken),
                    },
                },
            );

            await this.telemetry
                .trackApiCall(
                    `${this.apiBaseUrl}/components/${component.id}`,
                    "GET",
                    fetchPromise,
                    this.authToken
                )
                .then((response) => {
                    if (response.ok) {
                        return response.json();
                    }
                    // Handle 401 specially - trigger auth modal
                    if (response.status === 401) {
                        this.handleAuthenticationRequired();
                        return null;
                    }
                    // Log other errors but don't throw
                    console.error(`API Error: ${response.status} ${response.statusText}`);
                    this.telemetry.logError("API Error", {
                        url: `${this.apiBaseUrl}/components/${component.id}`,
                        status: response.status,
                        statusText: response.statusText
                    });
                    this.telemetry.showToast(
                        "Load Error",
                        `Failed to load component details (${response.status})`,
                        this.telemetry.levels.ERROR
                    );
                    return null;
                })
                .then((data) => {
                    if (data) {
                        this.componentDetails = data;
                    } else {
                        this.componentDetails = null;
                    }
                })
                .catch((error) => {
                    // Network errors
                    this.telemetry.logError(
                        "Failed to fetch component details",
                        error,
                    );
                    this.componentDetails = null;
                    console.error("Network error fetching component details:", error);
                })
                .finally(() => {
                    this.loadingDetails = false;
                });
        },

        async selectResource(component, resource) {
            this.telemetry.trackInteraction("select_resource", {
                componentId: component.id,
                resource: resource.key,
            });

            this.selectedItem = {
                type: "resource",
                key: resource.key,
                value: resource.value,
                componentId: component.id,
                componentName: component.name || component.id,
            };
            this.componentDetails = null;
            this.resourceData = null;
            this.loadingDetails = true;

            // Fetch data from the resource endpoint
            const fetchPromise = fetch(resource.value, {
                headers: {
                    "Content-Type": "application/json",
                    ...AuthService.getAuthHeader(this.authToken),
                },
            });

            await this.telemetry
                .trackApiCall(resource.value, "GET", fetchPromise, this.authToken)
                .then((response) => {
                    if (response.ok) {
                        return response.json();
                    }
                    // Handle 401 specially - trigger auth modal
                    if (response.status === 401) {
                        this.handleAuthenticationRequired();
                        return null;
                    }
                    // Log other errors but don't throw
                    console.error(`API Error: ${response.status} ${response.statusText}`);
                    this.telemetry.logError("API Error", {
                        url: resource.value,
                        status: response.status,
                        statusText: response.statusText
                    });
                    this.telemetry.showToast(
                        "Load Error",
                        `Failed to load resource data (${response.status})`,
                        this.telemetry.levels.ERROR
                    );
                    return null;
                })
                .then((data) => {
                    if (data) {
                        this.resourceData = data;
                    } else {
                        this.resourceData = null;
                    }
                })
                .catch((error) => {
                    // Network errors
                    this.telemetry.logError(
                        "Failed to fetch resource data",
                        error,
                    );
                    this.resourceData = null;
                    console.error("Network error fetching resource data:", error);
                })
                .finally(() => {
                    this.loadingDetails = false;
                });
        },

        formatResourceName(name) {
            // Convert snake_case to Title Case
            return name
                .split("_")
                .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
                .join(" ")
                .replace(/-/g, " ");
        },

        getResourceIcon(resourceKey) {
            const iconMap = {
                data: "bi bi-database",
                "data-list": "bi bi-database",
                data_list: "bi bi-database",
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
            return iconMap[resourceKey] || "bi bi-circle";
        },

        formatExtensionKey(key) {
            // Convert snake_case or camelCase to Title Case
            return key
                .replace(/([A-Z])/g, " $1")
                .replace(/_/g, " ")
                .split(" ")
                .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
                .join(" ")
                .trim();
        },

        formatExtensionValue(value) {
            if (value === null || value === undefined) {
                return "N/A";
            }
            if (typeof value === "object") {
                if (Array.isArray(value)) {
                    return value.join(", ");
                }
                return JSON.stringify(value);
            }
            return value.toString();
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
            await this.fetchComponents();
        },

        showVersionPaneOnHover() {
            this.isBannerHovered = true;
            this.showVersionPane = true;
            this.$nextTick(() => {
                this.positionVersionPane();
            });
        },

        hideVersionPaneOnLeave() {
            this.isBannerHovered = false;
            // Small delay to allow moving to the pane
            setTimeout(() => {
                if (!this.isPaneHovered && !this.isBannerHovered) {
                    this.showVersionPane = false;
                }
            }, 150);
        },

        keepVersionPaneOpen() {
            this.isPaneHovered = true;
        },

        positionVersionPane() {
            const serverBanner = document.querySelector(".server-banner");
            const versionPane = document.querySelector(".version-pane");

            if (serverBanner && versionPane) {
                const rect = serverBanner.getBoundingClientRect();
                versionPane.style.top = rect.top + "px";
                versionPane.style.left = rect.right + 8 + "px";
            }
        },

        hidePaneOnLeave() {
            this.isPaneHovered = false;
            setTimeout(() => {
                if (!this.isPaneHovered && !this.isBannerHovered) {
                    this.showVersionPane = false;
                }
            }, 150);
        },

        selectVersionFromPane(version) {
            this.selectedVersion = version.base_uri;
            this.showVersionPane = false;
            this.onVersionChange();
        },

        formatVendorInfo(vendorInfo) {
            if (!vendorInfo) return [];
            if (typeof vendorInfo === "object") {
                return Object.entries(vendorInfo);
            }
            return [];
        },

        clearSelection() {
            this.telemetry.trackInteraction("clear_selection");
            this.selectedItem = null;
            this.componentDetails = null;
            this.resourceData = null;
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

            if (componentId) {
                // Find the component in our list
                const component = this.componentsWithResources.find(
                    (c) => c.id === componentId,
                );
                if (component) {
                    // Expand the component to show its resources
                    this.expandedComponents[componentId] = true;

                    // Select the component
                    this.selectedItem = {
                        type: "component",
                        id: componentId,
                        name: componentName,
                    };
                    this.componentDetails = null;
                    this.resourceData = null;
                    this.loadingDetails = true;

                    const fetchPromise = fetch(
                        `${this.apiBaseUrl}/components/${componentId}`,
                        {
                            headers: {
                                "Content-Type": "application/json",
                                ...AuthService.getAuthHeader(this.authToken),
                            },
                        },
                    );

                    await this.telemetry
                        .trackApiCall(
                            `${this.apiBaseUrl}/components/${componentId}`,
                            "GET",
                            fetchPromise,
                            this.authToken
                        )
                        .then((response) => {
                            if (response.ok) {
                                return response.json();
                            }
                            // Handle 401 specially - trigger auth modal
                            if (response.status === 401) {
                                this.handleAuthenticationRequired();
                                return null;
                            }
                            // Log other errors but don't throw
                            console.error(`API Error: ${response.status} ${response.statusText}`);
                            this.telemetry.logError("API Error", {
                                url: `${this.apiBaseUrl}/components/${componentId}`,
                                status: response.status,
                                statusText: response.statusText
                            });
                            return null;
                        })
                        .then((data) => {
                            if (data) {
                                this.componentDetails = data;
                            } else {
                                this.componentDetails = null;
                            }
                        })
                        .catch((error) => {
                            // Network errors
                            this.telemetry.logError(
                                "Failed to fetch component details",
                                error,
                            );
                            this.componentDetails = null;
                            console.error("Network error fetching component details:", error);
                        })
                        .finally(() => {
                            this.loadingDetails = false;
                        });
                }
            }
        },

        hideTelemetryToast() {
            const toast = document.getElementById("telemetry-toast");
            if (toast) {
                toast.classList.remove("show");
            }
        },

        // Keyboard navigation methods
        handleKeyDown(event) {
            const key = event.key;
            const ctrlOrCmd = event.ctrlKey || event.metaKey;

            // Global shortcuts
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

            // Tree navigation with arrow keys
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
                        // Let Tab work normally but track keyboard mode
                        this.keyboardMode = true;
                        break;
                }
            }
        },

        focusFilterInput() {
            const filterInput = document.querySelector('.filter-input');
            if (filterInput) {
                filterInput.focus();
                filterInput.select();
                this.keyboardMode = true;
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
                // Blur any focused element
                document.activeElement.blur();
                this.focusedIndex = -1;
                this.focusedResourceIndex = -1;
            }
        },

        navigateDown() {
            if (this.focusedIndex === -1) {
                this.focusedIndex = 0;
                this.focusedResourceIndex = -1;
            } else {
                const currentComponent = this.filteredComponents[this.focusedIndex];

                // If component is expanded and has resources, navigate to first resource
                if (this.expandedComponents[currentComponent.id] &&
                    currentComponent.resources &&
                    currentComponent.resources.length > 0 &&
                    this.focusedResourceIndex === -1) {
                    this.focusedResourceIndex = 0;
                } else if (this.focusedResourceIndex !== -1 &&
                          this.focusedResourceIndex < currentComponent.resources.length - 1) {
                    // Navigate to next resource
                    this.focusedResourceIndex++;
                } else {
                    // Move to next component
                    if (this.focusedIndex < this.filteredComponents.length - 1) {
                        this.focusedIndex++;
                        this.focusedResourceIndex = -1;
                    }
                }
            }
            this.scrollToFocused();
        },

        navigateUp() {
            if (this.focusedResourceIndex > 0) {
                // Navigate to previous resource
                this.focusedResourceIndex--;
            } else if (this.focusedResourceIndex === 0) {
                // Move back to component
                this.focusedResourceIndex = -1;
            } else if (this.focusedIndex > 0) {
                // Move to previous component
                this.focusedIndex--;
                const prevComponent = this.filteredComponents[this.focusedIndex];

                // If previous component is expanded, focus last resource
                if (this.expandedComponents[prevComponent.id] &&
                    prevComponent.resources &&
                    prevComponent.resources.length > 0) {
                    this.focusedResourceIndex = prevComponent.resources.length - 1;
                }
            }
            this.scrollToFocused();
        },

        expandFocused() {
            if (this.focusedIndex >= 0 && this.focusedResourceIndex === -1) {
                const component = this.filteredComponents[this.focusedIndex];
                if (component.resources && component.resources.length > 0) {
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
                    // Select resource
                    const resource = component.resources[this.focusedResourceIndex];
                    this.selectResource(component, resource);
                } else {
                    // Select component
                    this.selectComponent(component);
                }
            }
        },

        focusFirst() {
            this.focusedIndex = 0;
            this.focusedResourceIndex = -1;
            this.scrollToFocused();
        },

        focusLast() {
            this.focusedIndex = this.filteredComponents.length - 1;
            const lastComponent = this.filteredComponents[this.focusedIndex];

            // If last component is expanded, focus last resource
            if (this.expandedComponents[lastComponent.id] &&
                lastComponent.resources &&
                lastComponent.resources.length > 0) {
                this.focusedResourceIndex = lastComponent.resources.length - 1;
            } else {
                this.focusedResourceIndex = -1;
            }
            this.scrollToFocused();
        },

        scrollToFocused() {
            this.$nextTick(() => {
                let focusedElement;
                if (this.focusedIndex >= 0) {
                    const component = this.filteredComponents[this.focusedIndex];
                    if (this.focusedResourceIndex >= 0) {
                        focusedElement = document.querySelector(
                            `[data-resource-id="${component.id}-${this.focusedResourceIndex}"]`
                        );
                    } else {
                        focusedElement = document.querySelector(
                            `[data-component-id="${component.id}"]`
                        );
                    }
                }

                if (focusedElement) {
                    focusedElement.scrollIntoView({
                        behavior: 'smooth',
                        block: 'nearest'
                    });
                }
            });
        },

        isFocused(componentIndex, resourceIndex = -1) {
            return this.keyboardMode &&
                   this.focusedIndex === componentIndex &&
                   this.focusedResourceIndex === resourceIndex;
        },

        // Authentication methods
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
            
            // Check if token is expired
            if (AuthService.isTokenExpired(claims)) {
                this.telemetry.showToast(
                    "Token Expired",
                    "The provided token has already expired",
                    this.telemetry.levels.ERROR
                );
                return;
            }
            
            // Save token
            AuthService.saveToken(token, this.persistToken);
            
            // Update state
            this.authToken = token;
            this.authTokenClaims = claims;
            this.authTokenExpiry = claims.exp;
            this.authTokenInput = ""; // Clear input
            
            // Start expiry check
            this.startTokenExpiryCheck();
            
            this.telemetry.log(
                "Authentication Success",
                `Authenticated as ${claims.sub || 'User'}`
            );
            
            // Refresh data with authenticated requests
            this.fetchComponents();
        },
        
        logout() {
            AuthService.clearToken();
            this.authToken = null;
            this.authTokenClaims = null;
            this.authTokenExpiry = null;
            this.authTokenInput = "";
            this.persistToken = false;
            this.tokenExpiryWarning = false;
            
            // Stop expiry check
            if (this.tokenExpiryCheckInterval) {
                clearInterval(this.tokenExpiryCheckInterval);
                this.tokenExpiryCheckInterval = null;
            }
            
            this.telemetry.log("Logout", "User logged out");
            
            // Refresh data (will make unauthenticated requests)
            this.fetchComponents();
        },
        
        handleAuthenticationRequired() {
            // Called when a 401 response is received
            this.showAuthModal = true;
            if (this.authToken) {
                this.telemetry.showToast(
                    "Authentication Failed",
                    "Your token may have expired. Please re-authenticate.",
                    this.telemetry.levels.WARNING
                );
            }
        },
        
        startTokenExpiryCheck() {
            // Clear existing interval
            if (this.tokenExpiryCheckInterval) {
                clearInterval(this.tokenExpiryCheckInterval);
            }
            
            // Check token expiry periodically
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
            
            const rtf = new Intl.RelativeTimeFormat('en', { numeric: 'auto' });
            const now = Math.floor(Date.now() / 1000);
            const diff = this.authTokenExpiry - now;
            
            if (diff < 0) {
                return 'Expired';
            } else if (diff < 60) {
                return rtf.format(diff, 'second');
            } else if (diff < 3600) {
                return rtf.format(Math.floor(diff / 60), 'minute');
            } else if (diff < 86400) {
                return rtf.format(Math.floor(diff / 3600), 'hour');
            } else {
                return rtf.format(Math.floor(diff / 86400), 'day');
            }
        },
        
        loadAuthToken() {
            // Load token from storage on mount
            const token = AuthService.loadToken();
            if (token) {
                const claims = AuthService.parseJWT(token);
                if (claims && !AuthService.isTokenExpired(claims)) {
                    this.authToken = token;
                    this.authTokenClaims = claims;
                    this.authTokenExpiry = claims.exp;
                    this.persistToken = localStorage.getItem(AuthService.TOKEN_KEY + '_persist') === 'true';
                    this.startTokenExpiryCheck();
                } else {
                    // Clear expired token
                    AuthService.clearToken();
                }
            }
        },
    },
    mounted() {
        // Initialize telemetry
        this.telemetry.init();

        // Log initial page load
        this.telemetry.trackInteraction("page_load", {
            url: window.location.href,
            referrer: document.referrer,
        });

        // Load authentication token if available
        this.loadAuthToken();

        // Fetch initial data
        this.fetchVersionInfo();
        this.fetchComponents();
        
        // Expose app instance for auth handling
        window.vueApp = this;

        // Setup keyboard event listeners
        document.addEventListener('keydown', this.handleKeyDown.bind(this));

        // Track mouse usage to disable keyboard mode
        document.addEventListener('mousedown', () => {
            this.keyboardMode = false;
            this.focusedIndex = -1;
            this.focusedResourceIndex = -1;
        });

        // Log telemetry summary every 30 seconds (for debugging)
        setInterval(() => {
            const summary = this.telemetry.getSummary();
            console.log("[Telemetry] Summary:", summary);
        }, 30000);
    },
    beforeUnmount() {
        // Log session end
        this.telemetry.trackInteraction("page_unload", {
            sessionDuration: performance.now(),
        });

        // Export metrics for debugging
        console.log(
            "[Telemetry] Final metrics:",
            this.telemetry.exportMetrics(),
        );
    },
};

// Create and configure the app
const app = createApp(appConfig);

// Set up error handler if possible
try {
    if (app && app.config) {
        app.config.errorHandler = (err, vm, info) => {
            console.error("Vue Error:", err, info);
            // We'll log to telemetry after mount when it's available
        };
    }
} catch (e) {
    console.warn("Could not set Vue error handler:", e);
}

// Mount the app
app.mount("#app");
