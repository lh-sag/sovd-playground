<script setup lang="ts">
const emit = defineEmits<{
    close: [];
}>();
</script>

<template>
    <div class="keyboard-help-modal" @click.self="emit('close')">
        <div class="keyboard-help-content">
            <div class="keyboard-help-header">
                <h5>Keyboard Shortcuts</h5>
                <button type="button" class="btn-close" @click="emit('close')"></button>
            </div>
            <div class="keyboard-help-body">
                <div class="shortcut-section">
                    <h6>Navigation</h6>
                    <div class="shortcut-item">
                        <kbd>↑</kbd> <kbd>↓</kbd>
                        <span>Navigate up/down in tree</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>←</kbd> <kbd>→</kbd>
                        <span>Collapse/expand components</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Enter</kbd>
                        <span>Select focused item</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Home</kbd> <kbd>End</kbd>
                        <span>Jump to first/last item</span>
                    </div>
                </div>
                <div class="shortcut-section">
                    <h6>Global</h6>
                    <div class="shortcut-item">
                        <kbd>/</kbd> or <kbd>Ctrl</kbd>+<kbd>K</kbd>
                        <span>Focus filter input</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Esc</kbd>
                        <span>Clear selection/close dialogs</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>?</kbd>
                        <span>Show this help</span>
                    </div>
                    <div class="shortcut-item">
                        <kbd>Tab</kbd>
                        <span>Navigate between panels</span>
                    </div>
                </div>
            </div>
            <div class="keyboard-help-footer">
                <small class="text-muted">Press <kbd>Esc</kbd> to close</small>
            </div>
        </div>
    </div>
</template>

<style scoped>
.keyboard-help-modal {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 10000;
    animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
    from {
        opacity: 0;
    }
    to {
        opacity: 1;
    }
}

.keyboard-help-content {
    background: var(--bg-primary);
    border-radius: 8px;
    width: 90%;
    max-width: 600px;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
    animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
    from {
        transform: translateY(20px);
        opacity: 0;
    }
    to {
        transform: translateY(0);
        opacity: 1;
    }
}

.keyboard-help-header {
    padding: 1.25rem;
    border-bottom: 1px solid var(--border-color);
    display: flex;
    justify-content: space-between;
    align-items: center;
}

.keyboard-help-header h5 {
    margin: 0;
    font-weight: 600;
    color: var(--text-primary);
}

.keyboard-help-body {
    padding: 1.25rem;
    overflow-y: auto;
    flex: 1;
}

.shortcut-section {
    margin-bottom: 1.5rem;
}

.shortcut-section:last-child {
    margin-bottom: 0;
}

.shortcut-section h6 {
    font-size: 0.875rem;
    font-weight: 600;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin-bottom: 0.75rem;
}

.shortcut-item {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 0;
    border-bottom: 1px solid var(--border-color);
}

@media (prefers-contrast: high) {
    .shortcut-item kbd {
        border-width: 2px;
    }
}

.shortcut-item:last-child {
    border-bottom: none;
}

.shortcut-item kbd {
    background: linear-gradient(to bottom, var(--bg-secondary), var(--border-color));
    border: 1px solid var(--border-dark);
    border-radius: 4px;
    padding: 0.25rem 0.5rem;
    font-size: 0.875rem;
    font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
    box-shadow: 0 2px 0 var(--border-dark);
    min-width: 2rem;
    text-align: center;
    color: var(--text-primary);
}

.shortcut-item span {
    flex: 1;
    color: var(--text-primary);
    font-size: 0.9rem;
}

.keyboard-help-footer {
    padding: 1rem 1.25rem;
    border-top: 1px solid var(--border-color);
    text-align: center;
}
</style>
