use pds::{Change, ChangeId, ChangeMetadata, HistoryDag, Operation, SyncEngine, SyncMessage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut node_a = HistoryDag::new();
    let mut node_b = HistoryDag::new();

    let root_id = ChangeId::new();
    let root_change = Change::new(
        root_id,
        vec![],
        ChangeMetadata::new("device-init"),
        vec![Operation::CreateEntity {
            entity_id: pds::EntityId::new(),
            entity_type: Some("root".into()),
        }],
    );
    node_a.insert(root_change.clone())?;
    node_b.insert(root_change)?;

    let a_id = ChangeId::new();
    node_a.insert(Change::new(
        a_id,
        vec![root_id],
        ChangeMetadata::new("device-a"),
        vec![],
    ))?;

    let b_id = ChangeId::new();
    node_b.insert(Change::new(
        b_id,
        vec![root_id],
        ChangeMetadata::new("device-b"),
        vec![],
    ))?;

    println!("Węzeł A ma heads: {:?}", node_a.heads());
    println!("Węzeł B ma heads: {:?}", node_b.heads());

    let req = SyncEngine::create_request(&node_a);
    let resp = match req {
        SyncMessage::SyncRequest { known_heads } => {
            SyncEngine::handle_request(&node_b, &known_heads)
        }
        _ => unreachable!(),
    };

    if let SyncMessage::SyncResponse {
        missing_changes, ..
    } = resp
    {
        let count = SyncEngine::apply_response(&mut node_a, missing_changes)?;
        println!("Do węzła A zaaplikowano brakujących zmian: {}", count);
    }

    println!(
        "Po synchronizacji węzeł A posiada heads: {:?}",
        node_a.heads()
    );
    Ok(())
}
