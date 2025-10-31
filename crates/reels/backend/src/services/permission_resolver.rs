//! Implements "Most Permissive Wins" permission resolution logic for collection hierarchies.
//!
//! This service resolves effective permissions by checking direct object permissions first,
//! then parent creative permissions, then collection permissions. It returns the most
//! permissive access level found (editor > viewer) to support collection sharing workflows
//! where users should get the best access available through any path in the hierarchy.

use crate::db::shares::AccessLevel;
use crate::queries::collections::get_collection_hierarchy::CollectionHierarchy;
use std::collections::HashMap;
use uuid::Uuid;

pub fn resolve_effective_permission(
    permissions_map: &HashMap<Uuid, AccessLevel>,
    hierarchy: &CollectionHierarchy,
    target_object_id: Uuid,
) -> Option<AccessLevel> {
    // Check direct permission on target object first
    if let Some(access_level) = permissions_map.get(&target_object_id) {
        return Some(*access_level);
    }

    // If target is an asset or document, check if it belongs to any creative in this collection
    let mut parent_creative_permissions = Vec::new();
    for creative_id in &hierarchy.creative_ids {
        if let Some(access_level) = permissions_map.get(creative_id) {
            parent_creative_permissions.push(*access_level);
        }
    }

    // Check permission on collection itself
    let mut collection_permission = None;
    if let Some(access_level) = permissions_map.get(&hierarchy.collection_id) {
        collection_permission = Some(*access_level);
    }

    // Combine all permissions and return most permissive (editor > viewer)
    let mut all_permissions = parent_creative_permissions;
    if let Some(perm) = collection_permission {
        all_permissions.push(perm);
    }

    // Return the most permissive permission found
    all_permissions.into_iter().max_by_key(|access_level| {
        match access_level {
            AccessLevel::Editor => 2,
            AccessLevel::Viewer => 1,
        }
    })
}

pub fn resolve_collection_member_permissions(
    permissions_map: &HashMap<Uuid, AccessLevel>,
    hierarchy: &CollectionHierarchy,
) -> HashMap<Uuid, AccessLevel> {
    let mut result = HashMap::new();

    // Resolve permissions for each creative in the collection
    for creative_id in &hierarchy.creative_ids {
        if let Some(permission) = resolve_effective_permission(permissions_map, hierarchy, *creative_id) {
            result.insert(*creative_id, permission);
        }
    }

    // Resolve permissions for each asset
    for asset_id in &hierarchy.asset_ids {
        if let Some(permission) = resolve_effective_permission(permissions_map, hierarchy, *asset_id) {
            result.insert(*asset_id, permission);
        }
    }

    // Resolve permissions for each document
    for document_id in &hierarchy.document_ids {
        if let Some(permission) = resolve_effective_permission(permissions_map, hierarchy, *document_id) {
            result.insert(*document_id, permission);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    //! Tests for permission_resolver.

    use super::*;

    #[test]
    fn test_direct_permission_takes_precedence() {
        let mut permissions = HashMap::new();
        let target_id = Uuid::new_v4();
        permissions.insert(target_id, AccessLevel::Editor);

        let hierarchy = CollectionHierarchy {
            collection_id: Uuid::new_v4(),
            creative_ids: vec![],
            asset_ids: vec![],
            document_ids: vec![],
        };

        let result = resolve_effective_permission(&permissions, &hierarchy, target_id);
        assert_eq!(result, Some(AccessLevel::Editor));
    }

    #[test]
    fn test_resolve_direct_permission_priority() {
        // Test that direct permissions on target objects take priority
        let target_id = Uuid::new_v4();
        let collection_id = Uuid::new_v4();
        
        let mut permissions = HashMap::new();
        permissions.insert(target_id, AccessLevel::Viewer);
        permissions.insert(collection_id, AccessLevel::Editor);
        
        let hierarchy = CollectionHierarchy {
            collection_id,
            creative_ids: vec![],
            asset_ids: vec![target_id],
            document_ids: vec![],
        };
        
        let result = resolve_effective_permission(&permissions, &hierarchy, target_id);
        assert_eq!(result, Some(AccessLevel::Viewer), "Direct permission should take priority over collection permission");
    }

    #[test]
    fn test_resolve_parent_creative_permission() {
        // Test that parent creative permissions are checked when no direct permission exists
        let asset_id = Uuid::new_v4();
        let creative_id = Uuid::new_v4();
        let collection_id = Uuid::new_v4();
        
        let mut permissions = HashMap::new();
        permissions.insert(creative_id, AccessLevel::Editor);
        
        let hierarchy = CollectionHierarchy {
            collection_id,
            creative_ids: vec![creative_id],
            asset_ids: vec![asset_id],
            document_ids: vec![],
        };
        
        let result = resolve_effective_permission(&permissions, &hierarchy, asset_id);
        assert_eq!(result, Some(AccessLevel::Editor), "Should inherit permission from parent creative");
    }

    #[test]
    fn test_resolve_collection_permission_fallback() {
        // Test that collection permissions are used as final fallback
        let asset_id = Uuid::new_v4();
        let collection_id = Uuid::new_v4();
        
        let mut permissions = HashMap::new();
        permissions.insert(collection_id, AccessLevel::Viewer);
        
        let hierarchy = CollectionHierarchy {
            collection_id,
            creative_ids: vec![],
            asset_ids: vec![asset_id],
            document_ids: vec![],
        };
        
        let result = resolve_effective_permission(&permissions, &hierarchy, asset_id);
        assert_eq!(result, Some(AccessLevel::Viewer), "Should inherit permission from collection as fallback");
    }

    #[test]
    fn test_resolve_no_permission_returns_none() {
        // Test that missing permissions return None
        let asset_id = Uuid::new_v4();
        let collection_id = Uuid::new_v4();
        
        let permissions = HashMap::new();
        
        let hierarchy = CollectionHierarchy {
            collection_id,
            creative_ids: vec![],
            asset_ids: vec![asset_id],
            document_ids: vec![],
        };
        
        let result = resolve_effective_permission(&permissions, &hierarchy, asset_id);
        assert_eq!(result, None, "Should return None when no permissions exist");
    }
}
