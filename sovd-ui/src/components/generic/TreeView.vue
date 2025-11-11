<script setup lang="ts">
import { computed, provide, ref, useId, useTemplateRef, watch } from 'vue';
import { telemetryService } from '../../services/TelemetryService';
import { NodePath, TreeViewContextInjectionKey } from './TreeViewContext';
import TreeViewNode from './TreeViewNode.vue';

const props = defineProps<{
    canExpand: (path: NodePath) => boolean;
    getChildren: (path: NodePath) => string[];
}>();

const emit = defineEmits<{
    focusBefore: [];
    focusAfter: [];
}>();

const idPrefix = useId();

const selectedPath = defineModel<NodePath | undefined>();

const focusedNode = ref<NodePath | null>(null);

const treeViewElement = useTemplateRef('tree-view');

const expandedNodes = ref(new Set<string>());

function isExpanded(path: NodePath): boolean {
    const key = nodePathToString(path);
    return expandedNodes.value.has(key);
}

function expand(path: NodePath) {
    const key = nodePathToString(path);
    expandedNodes.value.add(key);
}

function collapse(path: NodePath) {
    const key = nodePathToString(path);
    expandedNodes.value.delete(key);
}

function toggleExpand(path: NodePath) {
    if (isExpanded(path)) {
        collapse(path);
    } else {
        expand(path);
    }
}

function nodePathToString(path: NodePath): string {
    return path.join('/');
}

function hasChildren(path: NodePath): boolean {
    return props.getChildren(path).length > 0;
}

function expandPath(path: NodePath) {
    const parentPath = getParentPath(path);
    if (parentPath.length >= 1) {
        expandPath(parentPath);
    }
    expand(path);
}

// expand the selected node
watch(
    selectedPath,
    (selectedPath) => {
        if (selectedPath) {
            expandPath(selectedPath);
        }
        focusedNode.value = selectedPath ?? null;
    },
    { immediate: true },
);

function arePathsEqual(path1: NodePath, path2: NodePath): boolean {
    if (path1.length !== path2.length) return false;
    for (let i = 0; i < path1.length; i++) {
        if (path1[i] !== path2[i]) return false;
    }
    return true;
}

function getParentPath(path: NodePath): NodePath {
    if (path.length <= 1) return [];
    return path.slice(0, -1);
}

function getFirstVisibleNode(): NodePath | null {
    const rootPaths = props.getChildren([]);
    if (rootPaths.length === 0) return null;
    return [rootPaths[0]];
}

function getLastVisibleNode(): NodePath | null {
    const rootPaths = props.getChildren([]);
    if (rootPaths.length === 0) return null;
    const lastRootNode = rootPaths[rootPaths.length - 1];
    return getLastVisibleDescendent([lastRootNode]);
}

function getLastVisibleDescendent(path: NodePath): NodePath | null {
    if (!isExpanded(path)) return path;

    const childPaths = props.getChildren(path);
    if (childPaths.length === 0) return path;

    return getLastVisibleDescendent([...path, childPaths[childPaths.length - 1]]);
}

function getPrevVisibleNode(path: NodePath): NodePath | null {
    const nodeKey = path[path.length - 1];
    const parentPath = getParentPath(path);
    const siblingPaths = props.getChildren(parentPath);
    const currentIndex = siblingPaths.indexOf(nodeKey);
    if (currentIndex > 0) {
        const prevSiblingPath = siblingPaths[currentIndex - 1];
        return getLastVisibleDescendent([...parentPath, prevSiblingPath]);
    } else if (parentPath.length > 0) {
        return parentPath;
    } else {
        return null;
    }
}

function getNextVisibleNode(path: NodePath): NodePath | null {
    if (isExpanded(path)) {
        const childPaths = props.getChildren(path);
        if (childPaths.length > 0) {
            return [...path, childPaths[0]];
        }
    }

    while (path.length > 0) {
        const nodeKey = path[path.length - 1];
        const parentPath = getParentPath(path);
        const siblingPaths = props.getChildren(parentPath);
        const currentIndex = siblingPaths.indexOf(nodeKey);
        if (currentIndex !== -1 && currentIndex + 1 < siblingPaths.length) {
            return [...parentPath, siblingPaths[currentIndex + 1]];
        }

        path = parentPath;
    }

    return null;
}

function isSelected(path: NodePath): boolean {
    if (!selectedPath.value) return false;
    return arePathsEqual(path, selectedPath.value);
}

function isFocused(path: NodePath): boolean {
    if (!focusedNode.value) return false;
    return arePathsEqual(path, focusedNode.value);
}

function selectNode(path: NodePath) {
    if (isSelected(path)) {
        toggleExpand(path);
    } else {
        telemetryService.trackInteraction('select_node', { path });
        selectedPath.value = path;
    }
}

function selectFocusedNode() {
    if (!focusedNode.value) return;
    selectNode(focusedNode.value);
}

function getNodeElementId(path: NodePath): string {
    return `${idPrefix}-node-${path.join('/')}`;
}

function getNodeElement(path: NodePath): HTMLElement | undefined {
    const elementId = getNodeElementId(path);
    const element = document.getElementById(elementId);
    if (!(element instanceof HTMLElement)) return undefined;
    return element;
}

function focusPrevNode() {
    if (!focusedNode.value) return;
    const prevPath = getPrevVisibleNode(focusedNode.value);
    if (prevPath) {
        focusNode(prevPath);
    } else {
        // notify parent that focus should be moved further up
        emit('focusBefore');
    }
}

function focusNextNode() {
    if (!focusedNode.value) return;
    const nextPath = getNextVisibleNode(focusedNode.value);
    if (nextPath) {
        focusNode(nextPath);
    } else {
        // notify parent that focus should be moved further down
        emit('focusAfter');
    }
}

function onRight() {
    if (!focusedNode.value) return;

    // expand node if not already expanded
    if (!isExpanded(focusedNode.value)) {
        expand(focusedNode.value);
        return;
    }

    // move focus to first child if available
    const childPaths = props.getChildren(focusedNode.value);
    if (childPaths.length === 0) return;
    focusNode([...focusedNode.value, childPaths[0]]);
}

function onLeft() {
    if (focusedNode.value === null) return;

    // collapse node if expanded
    if (isExpanded(focusedNode.value)) {
        collapse(focusedNode.value);
        return;
    }

    // if focus is on a child, move it back to the parent
    const parentPath = getParentPath(focusedNode.value);
    if (parentPath.length >= 1) {
        focusNode(parentPath);
    }
}

function focusFirstNode() {
    const node = getFirstVisibleNode();
    if (!node) return;
    focusNode(node);
}

function focusLastNode() {
    const node = getLastVisibleNode();
    if (!node) return;
    focusNode(node);
}

function focusNode(path: NodePath) {
    const element = getNodeElement(path);
    if (!element) return;
    element.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    treeViewElement.value?.focus();
    focusedNode.value = path;
}

const activeDescendantId = computed(() => {
    if (!focusedNode.value) return undefined;
    return getNodeElementId(focusedNode.value);
});

provide(TreeViewContextInjectionKey, {
    isExpanded,
    getChildren: props.getChildren,
    canExpand: props.canExpand,
    hasChildren,
    isSelected,
    isFocused,
    selectNode,
    getNodeElementId,
});

defineExpose({
    focusFirstNode,
    focusLastNode,
});
</script>

<template>
    <ul
        ref="tree-view"
        role="tree"
        class="tree-view"
        tabindex="0"
        :aria-activedescendant="activeDescendantId"
        @keydown.up="focusPrevNode"
        @keydown.down="focusNextNode"
        @keydown.home="focusFirstNode"
        @keydown.end="focusLastNode"
        @keydown.left="onLeft"
        @keydown.right="onRight"
        @keydown.enter="selectFocusedNode"
    >
        <TreeViewNode v-for="id in getChildren([])" :key="id" :path="[id]" v-slot="{ path }">
            <slot :path />
        </TreeViewNode>
    </ul>
</template>

<style scoped>
.tree-view {
    list-style: none;
    padding-left: 0;
}

.tree-view:focus-visible {
    outline: none;
}

.tree-view:focus-visible :deep(.tree-view-node__label.focused) {
    outline: 2px solid var(--bs-primary);
    outline-offset: -2px;
}

.tree-view:focus-visible :deep(.tree-view-node__label:not(.selected).focused) {
    background-color: rgba(13, 110, 253, 0.05);
}

@media (prefers-contrast: high) {
    .tree-view:focus-visible :deep(.tree-view-node__label.focused) {
        outline-width: 3px;
        outline-style: solid;
    }
}
</style>
