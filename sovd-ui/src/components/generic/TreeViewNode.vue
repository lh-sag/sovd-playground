<script setup lang="ts">
import { NodePath, useTreeViewContext } from './TreeViewContext';

defineProps<{
    path: NodePath;
}>();

defineSlots<{
    default: { path: NodePath };
}>();

const tree = useTreeViewContext();
</script>

<template>
    <li class="tree-view-node">
        <div
            :id="tree.getNodeElementId(path)"
            class="tree-view-node__label"
            :class="{
                selected: tree.isSelected(path),
                focused: tree.isFocused(path),
            }"
            @click="tree.selectNode(path)"
            role="treeitem"
            :aria-expanded="tree.isExpanded(path)"
            :aria-selected="tree.isSelected(path)"
        >
            <template v-if="tree.canExpand(path)">
                <i
                    v-if="tree.hasChildren(path)"
                    class="bi bi-caret-right-fill expand-icon me-2"
                    :class="{ expanded: tree.isExpanded(path) }"
                ></i>
                <span v-else class="expand-spacer"></span>
            </template>
            <slot :path="path" />
        </div>

        <ul v-if="tree.isExpanded(path) && tree.getChildren(path).length > 0" class="tree-view-node__subtree">
            <TreeViewNode v-for="id in tree.getChildren(path)" :key="id" :path="[...path, id]" v-slot="{ path }">
                <slot :path="path" />
            </TreeViewNode>
        </ul>
    </li>
</template>

<style scoped>
.tree-view-node {
    user-select: none;
}

.tree-view-node__label {
    padding: 0.35rem 0.5rem;
    margin: 0.15rem 0;
    cursor: pointer;
    border-radius: 4px;
    transition:
        background-color 0.2s,
        color 0.2s;
    display: flex;
    align-items: center;
    font-size: 0.9rem;
}

.tree-view-node__label:hover {
    background-color: var(--bg-hover);
    color: var(--text-hover);
}

.tree-view-node__label.selected {
    background-color: var(--bg-selected);
    color: #000000;
}

.tree-view-node__label i {
    margin-right: 0.35rem;
    transition: transform 0.2s;
    font-size: 0.85rem;
}

.tree-view-node__label .expand-icon {
    margin-right: 0.2rem;
    font-size: 0.65rem;
    transition: transform 0.2s;
}

.tree-view-node__label .expand-icon.expanded {
    transform: rotate(90deg);
}

.tree-view-node__subtree {
    list-style: none;
    padding-left: 1.25rem;
    margin: 0;
}

.expand-spacer {
    width: 1rem;
    display: inline-block;
}
</style>
