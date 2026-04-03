export interface TreeNodeLite {
  id: number;
  parent_id?: number | null;
  children?: TreeNodeLite[];
}

export interface DropPlan {
  newParentId: number | null;
  insertIndex: number;
}

interface AntTreeDropInfo {
  node: { key: string | number; pos?: string };
  dropToGap: boolean;
  dropPosition: number;
}

function findParentId(
  tree: TreeNodeLite[],
  childId: number,
  parentId: number | null = null
): number | null {
  for (const node of tree) {
    if (node.id === childId) return parentId;
    const found = findParentId(node.children || [], childId, node.id ?? null);
    if (found !== null) return found;
  }
  return null;
}

function findNode(tree: TreeNodeLite[], targetId: number): TreeNodeLite | null {
  for (const node of tree) {
    if (node.id === targetId) return node;
    const found = findNode(node.children || [], targetId);
    if (found) return found;
  }
  return null;
}

function getSiblingsByParent(tree: TreeNodeLite[], parentId: number | null): TreeNodeLite[] {
  if (parentId === null) {
    return tree.filter((node) => (node.parent_id ?? null) === null);
  }
  const parent = findNode(tree, parentId);
  return parent ? [...(parent.children || [])] : [];
}

export function buildDropPlan(
  tree: TreeNodeLite[],
  dragId: number,
  dropId: number,
  info: AntTreeDropInfo
): DropPlan {
  if (!info.dropToGap) {
    // Drop on node: move into target as child and append to tail.
    const newParentId = dropId;
    const siblings = getSiblingsByParent(tree, newParentId).filter(
      (node) => node.id !== dragId
    );
    return { newParentId, insertIndex: siblings.length };
  }

  const newParentId = findParentId(tree, dropId);
  const siblings = getSiblingsByParent(tree, newParentId).filter(
    (node) => node.id !== dragId
  );
  const targetIndex = siblings.findIndex((node) => node.id === dropId);
  if (targetIndex < 0) {
    return { newParentId, insertIndex: siblings.length };
  }

  let relativeDropPosition = info.dropPosition;
  if (typeof info.node.pos === 'string' && info.node.pos.length > 0) {
    const nodePosIndex = Number(info.node.pos.split('-').pop() || 0);
    if (Number.isFinite(nodePosIndex)) {
      relativeDropPosition = info.dropPosition - nodePosIndex;
    }
  }
  const insertIndex =
    relativeDropPosition < 0 ? targetIndex : targetIndex + 1;

  return { newParentId, insertIndex };
}
