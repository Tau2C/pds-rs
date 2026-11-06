use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Permission {
    ReadEntities,
    WriteEntities,
    ReadBlobs,
    WriteBlobs,
    ReadHistory,
    WriteHistory,
    ManagePermissions,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppRegistration {
    pub app_id: String,
    pub name: String,
    pub permissions: BTreeSet<Permission>,
    pub revoked: bool,
}

impl AppRegistration {
    pub fn new(app_id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            name: name.into(),
            permissions: BTreeSet::new(),
            revoked: false,
        }
    }

    pub fn grant(&mut self, perm: Permission) {
        self.permissions.insert(perm);
    }

    pub fn revoke_permission(&mut self, perm: &Permission) {
        self.permissions.remove(perm);
    }

    pub fn revoke_app(&mut self) {
        self.revoked = true;
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum SecurityError {
    #[error("App not found: {0}")]
    AppNotFound(String),
    #[error("App access has been revoked: {0}")]
    AppRevoked(String),
    #[error("Permission denied for app {app_id}: requires {permission:?}")]
    PermissionDenied {
        app_id: String,
        permission: Permission,
    },
}

#[derive(Debug, Clone, Default)]
pub struct AccessController {
    trusted_devices: BTreeSet<String>,
    apps: BTreeMap<String, AppRegistration>,
}

impl AccessController {
    pub fn new() -> Self {
        Self {
            trusted_devices: BTreeSet::new(),
            apps: BTreeMap::new(),
        }
    }

    pub fn add_trusted_device(&mut self, device_id: impl Into<String>) {
        self.trusted_devices.insert(device_id.into());
    }

    pub fn is_device_trusted(&self, device_id: &str) -> bool {
        self.trusted_devices.contains(device_id)
    }

    pub fn register_app(&mut self, app: AppRegistration) {
        self.apps.insert(app.app_id.clone(), app);
    }

    pub fn check_permission(&self, app_id: &str, perm: &Permission) -> Result<(), SecurityError> {
        let app = self
            .apps
            .get(app_id)
            .ok_or_else(|| SecurityError::AppNotFound(app_id.to_string()))?;

        if app.revoked {
            return Err(SecurityError::AppRevoked(app_id.to_string()));
        }

        if !app.permissions.contains(perm) {
            return Err(SecurityError::PermissionDenied {
                app_id: app_id.to_string(),
                permission: perm.clone(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_controller_permissions_and_revocation() {
        let mut ac = AccessController::new();
        ac.add_trusted_device("laptop-dev");
        assert!(ac.is_device_trusted("laptop-dev"));
        assert!(!ac.is_device_trusted("unknown-phone"));

        let mut app = AppRegistration::new("app:notes", "Notes Client");
        app.grant(Permission::ReadEntities);
        ac.register_app(app);

        assert!(
            ac.check_permission("app:notes", &Permission::ReadEntities)
                .is_ok()
        );
        assert!(matches!(
            ac.check_permission("app:notes", &Permission::WriteEntities),
            Err(SecurityError::PermissionDenied { .. })
        ));

        // Revoke application
        let mut app_revoked = ac.apps.get("app:notes").unwrap().clone();
        app_revoked.revoke_app();
        ac.register_app(app_revoked);

        assert!(matches!(
            ac.check_permission("app:notes", &Permission::ReadEntities),
            Err(SecurityError::AppRevoked(_))
        ));
    }
}
