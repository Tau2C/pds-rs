use crate::change::{Change, ChangeMetadata};
use crate::entity::Entity;
use crate::id::{ChangeId, EntityId, RelationId};
use crate::operation::Operation;

use crate::relation::Relation;
use crate::security::{AccessController, Permission, SecurityError};
use crate::storage::{DiskStorage, StorageError};
use crate::value::{PropertyName, PropertyValue};

#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error("Security error: {0}")]
    Security(#[from] SecurityError),
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Entity not found: {0}")]
    EntityNotFound(EntityId),
    #[error("Relation not found: {0}")]
    RelationNotFound(RelationId),
}

pub struct PdsClient<'a> {
    app_id: String,
    device_id: String,
    security: &'a AccessController,
    storage: &'a mut DiskStorage,
}

impl<'a> PdsClient<'a> {
    pub fn new(
        app_id: impl Into<String>,
        device_id: impl Into<String>,
        security: &'a AccessController,
        storage: &'a mut DiskStorage,
    ) -> Self {
        Self {
            app_id: app_id.into(),
            device_id: device_id.into(),
            security,
            storage,
        }
    }

    pub fn get_entity(&self, id: &EntityId) -> Result<Option<Entity>, ClientError> {
        self.security
            .check_permission(&self.app_id, &Permission::ReadEntities)?;
        Ok(self.storage.state().entities.get(id).cloned())
    }

    pub fn get_relation(&self, id: &RelationId) -> Result<Option<Relation>, ClientError> {
        self.security
            .check_permission(&self.app_id, &Permission::ReadEntities)?;
        Ok(self.storage.state().relations.get(id).cloned())
    }

    pub fn create_entity(
        &mut self,
        entity_type: Option<String>,
        properties: Vec<(PropertyName, PropertyValue)>,
    ) -> Result<EntityId, ClientError> {
        self.security
            .check_permission(&self.app_id, &Permission::WriteEntities)?;

        let entity_id = EntityId::new();
        let mut ops = vec![Operation::CreateEntity {
            entity_id,
            entity_type,
        }];

        for (name, val) in properties {
            ops.push(Operation::SetEntityProperty {
                entity_id,
                name,
                value: val,
            });
        }

        let parents: Vec<ChangeId> = self.storage.dag().heads().iter().copied().collect();
        let meta = ChangeMetadata::new(&self.device_id).with_author(&self.app_id);
        let change = Change::new(ChangeId::new(), parents, meta, ops);

        self.storage.append_change(change)?;
        Ok(entity_id)
    }

    pub fn create_relation(
        &mut self,
        relation_type: impl Into<String>,
        source: EntityId,
        target: EntityId,
    ) -> Result<RelationId, ClientError> {
        self.security
            .check_permission(&self.app_id, &Permission::WriteEntities)?;

        let relation_id = RelationId::new();
        let ops = vec![Operation::CreateRelation {
            relation_id,
            relation_type: relation_type.into(),
            source,
            target,
        }];

        let parents: Vec<ChangeId> = self.storage.dag().heads().iter().copied().collect();
        let meta = ChangeMetadata::new(&self.device_id).with_author(&self.app_id);
        let change = Change::new(ChangeId::new(), parents, meta, ops);

        self.storage.append_change(change)?;
        Ok(relation_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::AppRegistration;
    use std::fs;

    #[test]
    fn test_client_enforces_permissions() {
        let temp_dir = std::env::temp_dir().join(format!("pds_client_test_{}", EntityId::new()));
        let mut storage = DiskStorage::open(&temp_dir).unwrap();
        let mut security = AccessController::new();

        let mut app = AppRegistration::new("app:reader", "Reader App");
        app.grant(Permission::ReadEntities);
        security.register_app(app);

        // App reader tries to write entity - should fail
        let mut client = PdsClient::new("app:reader", "dev-1", &security, &mut storage);
        let result = client.create_entity(Some("note".into()), vec![]);
        assert!(matches!(result, Err(ClientError::Security(_))));

        // Grant write permission
        let mut app_writer = AppRegistration::new("app:writer", "Writer App");
        app_writer.grant(Permission::ReadEntities);
        app_writer.grant(Permission::WriteEntities);
        security.register_app(app_writer);

        let mut client_writer = PdsClient::new("app:writer", "dev-1", &security, &mut storage);
        let e_id = client_writer
            .create_entity(
                Some("note".into()),
                vec![("title".into(), PropertyValue::from("Witaj PDS"))],
            )
            .unwrap();

        let loaded = client_writer.get_entity(&e_id).unwrap().unwrap();
        assert_eq!(
            loaded.get_property(&"title".into()),
            Some(&PropertyValue::from("Witaj PDS"))
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
