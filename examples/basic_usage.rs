use pds::{
    AccessController, AppRegistration, DiskStorage, PdsClient, Permission, PropertyValue,
};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let store_dir = std::env::temp_dir().join("pds_example_basic");
    if store_dir.exists() {
        fs::remove_dir_all(&store_dir)?;
    }

    let mut storage = DiskStorage::open(&store_dir)?;
    let mut security = AccessController::new();

    let mut app = AppRegistration::new("app:contact_book", "Contact Book");
    app.grant(Permission::ReadEntities);
    app.grant(Permission::WriteEntities);
    security.register_app(app);

    let mut client = PdsClient::new("app:contact_book", "device-laptop", &security, &mut storage);

    let contact_id = client.create_entity(
        Some("contact".into()),
        vec![
            ("name".into(), PropertyValue::from("Anna Nowak")),
            ("email".into(), PropertyValue::from("anna@example.com")),
        ],
    )?;

    let org_id = client.create_entity(
        Some("organization".into()),
        vec![("title".into(), PropertyValue::from("Acme Corp"))],
    )?;

    let rel_id = client.create_relation("employed_at", contact_id, org_id)?;

    println!("Encja kontaktu: {}", contact_id);
    println!("Encja organizacji: {}", org_id);
    println!("Identyfikator relacji: {}", rel_id);

    let fetched_contact = client.get_entity(&contact_id)?.expect("Encja istnieje");
    println!(
        "Pobrana nazwa kontaktu: {:?}",
        fetched_contact.get_property(&"name".into())
    );

    let _ = fs::remove_dir_all(&store_dir);
    Ok(())
}
