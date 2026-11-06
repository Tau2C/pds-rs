use pds::{BlobId, BlobStore};
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = std::env::temp_dir().join("pds_example_blobs");
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    let mut store = BlobStore::new(&temp_dir)?;

    let photo_bytes = b"\x89PNG\r\n\x1a\nSample binary PNG payload data";
    let blob_id_1 = BlobId::new();
    let blob_id_2 = BlobId::new();

    let desc1 = store.put(blob_id_1, photo_bytes, Some("image/png".into()))?;
    let desc2 = store.put(blob_id_2, photo_bytes, Some("image/png".into()))?;

    println!("Zapisano zasób 1: {:?}", desc1);
    println!("Zapisano zasób 2: {:?}", desc2);
    println!("Deduplikacja skrótu SHA-256: {}", desc1.sha256 == desc2.sha256);

    let read_bytes = store.get_data(&blob_id_1)?;
    assert_eq!(&read_bytes[..], photo_bytes);
    println!("Zweryfikowano integralność i odczytano bajty poprawnie.");

    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}
