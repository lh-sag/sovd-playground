import { inject, InjectionKey } from 'vue';

export const TreeViewContextInjectionKey = Symbol() as InjectionKey<TreeViewContext>;

export type NodePath = string[];

export interface TreeViewContext {
    isExpanded: (path: NodePath) => boolean;
    getChildren: (path: NodePath) => string[];
    canExpand: (path: NodePath) => boolean;
    selectNode: (path: NodePath) => void;
    isSelected: (path: NodePath) => boolean;
    isFocused: (path: NodePath) => boolean;
    hasChildren: (path: NodePath) => boolean;
    getNodeElementId: (path: NodePath) => string;
}

export function useTreeViewContext(): TreeViewContext {
    const context = inject(TreeViewContextInjectionKey);
    if (!context) {
        throw new Error('TreeViewContext not provided');
    }
    return context;
}
