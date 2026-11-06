use pds::{
    Change, ChangeId, ChangeMetadata, DatasetState, EntityId, HistoryDag, Operation,
    PropertyName, PropertyValue, RelationId,
};

#[test]
fn test_dag_history_linear_and_branching() {
    let mut dag = HistoryDag::new();

    let c1_id = ChangeId::new();
    let c1 = Change::new(
        c1_id,
        vec![],
        ChangeMetadata::new("author1"),
        vec![Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("root".to_string()),
        }],
    );

    dag.insert(c1).expect("Failed to add c1");
    assert_eq!(dag.heads().iter().copied().collect::<Vec<_>>(), vec![c1_id]);

    let c2_id = ChangeId::new();
    let c2 = Change::new(
        c2_id,
        vec![c1_id],
        ChangeMetadata::new("author2"),
        vec![Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("child_branch_a".to_string()),
        }],
    );
    dag.insert(c2).expect("Failed to add c2");
    assert_eq!(dag.heads().iter().copied().collect::<Vec<_>>(), vec![c2_id]);

    // Branching from c1
    let c3_id = ChangeId::new();
    let c3 = Change::new(
        c3_id,
        vec![c1_id],
        ChangeMetadata::new("author3"),
        vec![Operation::CreateEntity {
            entity_id: EntityId::new(),
            entity_type: Some("child_branch_b".to_string()),
        }],
    );
    dag.insert(c3).expect("Failed to add c3");

    // Heads should now contain both c2 and c3
    let mut heads: Vec<ChangeId> = dag.heads().iter().copied().collect();
    heads.sort();
    let mut expected_heads = vec![c2_id, c3_id];
    expected_heads.sort();
    assert_eq!(heads, expected_heads);

    // Merge node combining c2 and c3
    let c4_id = ChangeId::new();
    let c4 = Change::new(
        c4_id,
        vec![c2_id, c3_id],
        ChangeMetadata::new("merge_author"),
        vec![],
    );
    dag.insert(c4).expect("Failed to add merge change c4");
    assert_eq!(dag.heads().iter().copied().collect::<Vec<_>>(), vec![c4_id]);

    let topo = dag.topological_order();
    assert_eq!(topo.len(), 4);
    assert_eq!(topo[0].id, c1_id);
    assert_eq!(topo[3].id, c4_id);
}

#[test]
fn test_dataset_state_projection_lifecycle() {
    let mut dag = HistoryDag::new();
    let entity_id = EntityId::new();
    let rel_id = RelationId::new();
    let target_id = EntityId::new();

    let c1_id = ChangeId::new();
    let c1 = Change::new(
        c1_id,
        vec![],
        ChangeMetadata::new("admin"),
        vec![
            Operation::CreateEntity {
                entity_id,
                entity_type: Some("account".to_string()),
            },
            Operation::SetEntityProperty {
                entity_id,
                name: "balance".into(),
                value: PropertyValue::Int(100),
            },
            Operation::CreateEntity {
                entity_id: target_id,
                entity_type: Some("device".to_string()),
            },
            Operation::CreateRelation {
                relation_id: rel_id,
                relation_type: "paired_to".to_string(),
                source: entity_id,
                target: target_id,
            },
        ],
    );
    dag.insert(c1).unwrap();

    let state = DatasetState::from_dag(&dag);
    assert_eq!(state.entities.len(), 2);
    assert_eq!(state.relations.len(), 1);

    let entity = state.entities.get(&entity_id).unwrap();
    assert_eq!(
        entity.get_property(&PropertyName::from("balance")),
        Some(&PropertyValue::Int(100))
    );

    // Now add mutation and deletion
    let c2_id = ChangeId::new();
    let c2 = Change::new(
        c2_id,
        vec![c1_id],
        ChangeMetadata::new("admin"),
        vec![
            Operation::SetEntityProperty {
                entity_id,
                name: "balance".into(),
                value: PropertyValue::Int(250),
            },
            Operation::DeleteRelation {
                relation_id: rel_id,
            },
        ],
    );
    dag.insert(c2).unwrap();

    let state2 = DatasetState::from_dag(&dag);
    assert_eq!(state2.relations.len(), 0);
    assert_eq!(
        state2
            .entities
            .get(&entity_id)
            .unwrap()
            .get_property(&PropertyName::from("balance")),
        Some(&PropertyValue::Int(250))
    );
}
