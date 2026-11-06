use pds::{Entity, EntityId, Operation, PropertyName, PropertyValue, Relation, RelationId};

#[test]
fn test_entity_creation_and_properties_manipulation() {
    let id = EntityId::new();
    let mut entity = Entity::with_type(id, "person");

    let name_prop: PropertyName = "user.name".into();
    let age_prop: PropertyName = "user.age".into();

    entity.set_property(name_prop.clone(), "Jan Kowalski");
    entity.set_property(age_prop.clone(), 42i64);

    assert_eq!(entity.id, id);
    assert_eq!(entity.entity_type.as_deref(), Some("person"));
    assert_eq!(
        entity.get_property(&name_prop),
        Some(&PropertyValue::Text("Jan Kowalski".to_string()))
    );
    assert_eq!(
        entity.get_property(&age_prop),
        Some(&PropertyValue::Int(42))
    );

    let removed = entity.remove_property(&age_prop);
    assert_eq!(removed, Some(PropertyValue::Int(42)));
    assert_eq!(entity.get_property(&age_prop), None);
}

#[test]
fn test_relation_creation() {
    let id = RelationId::new();
    let from = EntityId::new();
    let to = EntityId::new();
    let mut relation = Relation::new(id, "authored_by", from, to);

    let role_prop: PropertyName = "role".into();
    relation.set_property(role_prop.clone(), "author");

    assert_eq!(relation.id, id);
    assert_eq!(relation.source, from);
    assert_eq!(relation.target, to);
    assert_eq!(relation.relation_type, "authored_by");
    assert_eq!(
        relation.get_property(&role_prop),
        Some(&PropertyValue::Text("author".to_string()))
    );
}

#[test]
fn test_operations_variants() {
    let e_id = EntityId::new();
    let r_id = RelationId::new();

    let create_op = Operation::CreateEntity {
        entity_id: e_id,
        entity_type: Some("item".to_string()),
    };
    let set_prop_op = Operation::SetEntityProperty {
        entity_id: e_id,
        name: "item.price".into(),
        value: PropertyValue::Float(19.99),
    };
    let delete_prop_op = Operation::RemoveEntityProperty {
        entity_id: e_id,
        name: "item.price".into(),
    };
    let create_rel_op = Operation::CreateRelation {
        relation_id: r_id,
        relation_type: "related".to_string(),
        source: e_id,
        target: EntityId::new(),
    };
    let delete_rel_op = Operation::DeleteRelation {
        relation_id: r_id,
    };
    let delete_entity_op = Operation::DeleteEntity { entity_id: e_id };

    let operations = vec![
        create_op,
        set_prop_op,
        delete_prop_op,
        create_rel_op,
        delete_rel_op,
        delete_entity_op,
    ];

    let serialized = serde_json::to_string(&operations).expect("Serialization failed");
    let deserialized: Vec<Operation> =
        serde_json::from_str(&serialized).expect("Deserialization failed");
    assert_eq!(deserialized, operations);
}
