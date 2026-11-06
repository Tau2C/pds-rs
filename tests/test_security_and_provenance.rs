use pds::{
    AccessController, Activity, ActivityStatus, AppRegistration, BlobId, ChangeId, EntityId,
    Permission, ProvenanceTarget, SecurityError,
};

#[test]
fn test_access_controller_granular_permissions() {
    let mut controller = AccessController::new();

    controller.add_trusted_device("phone_device_1");
    assert!(controller.is_device_trusted("phone_device_1"));
    assert!(!controller.is_device_trusted("unknown_device"));

    let mut app = AppRegistration::new("app_health", "Health Tracker");
    app.grant(Permission::ReadEntities);
    app.grant(Permission::WriteEntities);

    controller.register_app(app);

    // Read and Write permitted
    assert!(controller.check_permission("app_health", &Permission::ReadEntities).is_ok());
    assert!(controller.check_permission("app_health", &Permission::WriteEntities).is_ok());

    // Blobs not permitted
    assert_eq!(
        controller.check_permission("app_health", &Permission::ReadBlobs),
        Err(SecurityError::PermissionDenied {
            app_id: "app_health".to_string(),
            permission: Permission::ReadBlobs,
        })
    );

    // Unknown app
    assert_eq!(
        controller.check_permission("unregistered_app", &Permission::ReadEntities),
        Err(SecurityError::AppNotFound("unregistered_app".to_string()))
    );
}

#[test]
fn test_app_revocation_lifecycle() {
    let mut controller = AccessController::new();
    let mut app = AppRegistration::new("app_notes", "Notes");
    app.grant(Permission::ReadEntities);
    app.grant(Permission::WriteEntities);

    controller.register_app(app);
    assert!(controller.check_permission("app_notes", &Permission::ReadEntities).is_ok());

    // Revoke app by registering revoked version
    let mut app_revoked = AppRegistration::new("app_notes", "Notes");
    app_revoked.grant(Permission::ReadEntities);
    app_revoked.revoke_app();
    controller.register_app(app_revoked);

    assert_eq!(
        controller.check_permission("app_notes", &Permission::ReadEntities),
        Err(SecurityError::AppRevoked("app_notes".to_string()))
    );
}

#[test]
fn test_activity_provenance_tracking() {
    let mut activity = Activity::new("ocr_processor", "1.2.0");
    assert_eq!(activity.status, ActivityStatus::Started);

    let input_blob = BlobId::new();
    let output_entity = EntityId::new();
    let change_id = ChangeId::new();

    activity.add_input(ProvenanceTarget::Blob(input_blob));
    activity.add_output(ProvenanceTarget::Entity(output_entity));
    activity.add_output(ProvenanceTarget::Change(change_id));
    activity.set_config("language", "pol+eng");

    assert_eq!(activity.inputs.len(), 1);
    assert_eq!(activity.outputs.len(), 2);
    assert_eq!(activity.configuration.get("language").map(String::as_str), Some("pol+eng"));

    activity.complete();
    assert_eq!(activity.status, ActivityStatus::Completed);
    assert!(activity.completed_at.is_some());
}
