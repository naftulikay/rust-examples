use anyhow::{Result, anyhow};
use futures::stream::StreamExt;
use futures::{AsyncReadExt, AsyncWriteExt};
use sha2::{Digest, Sha256};
use tracing_subscriber::EnvFilter;
use vfs::async_vfs::{
    AsyncMemoryFS as MemoryFS, AsyncOverlayFS as OverlayFS, AsyncPhysicalFS as PhysicalFS,
    AsyncVfsPath as VfsPath,
};

#[tokio::main]
async fn main() -> Result<()> {
    init_logging();
    let physical_fs = PhysicalFS::new(std::env::temp_dir());
    let memory_fs = MemoryFS::new();
    let fs: OverlayFS = OverlayFS::new(&[memory_fs.into(), physical_fs.into()]);

    let root: VfsPath = VfsPath::new(fs);

    // list files
    let entries = list(&root).await?;

    for found_path in &entries {
        tracing::debug!(
            path = root.join(found_path.as_str())?.as_str(),
            entry_type = if found_path.is_dir().await? {
                "dir"
            } else {
                "file"
            },
            "Found entry"
        );
    }

    // find the first file
    let mut found_file = None;

    for found_path in &entries {
        if found_path.is_file().await? {
            found_file = Some(found_path);
            break;
        }
    }

    if found_file.is_none() {
        return Err(anyhow!("Unable to find a file in /tmp"));
    }

    let overlaid_file = found_file.unwrap();
    let overlaid_checksum = hash(&overlaid_file).await?;

    let physical_fs = PhysicalFS::new(std::env::temp_dir());
    let physical_file = VfsPath::new(physical_fs).join(overlaid_file.as_str())?;
    let physical_checksum = hash(&physical_file).await?;

    tracing::info!(
        overlay = overlaid_checksum,
        physical = physical_checksum,
        "Compared digests between overlay and physical, they are {}",
        if overlaid_checksum.eq(&physical_checksum) {
            "equal"
        } else {
            "not equal"
        }
    );

    if overlaid_checksum.ne(&physical_checksum) {
        return Err(anyhow!("Discrepancy between files!"));
    }

    // now, write to the overlay and compare contents again
    tracing::info!("Writing to file in overlay");

    write(overlaid_file, "Hello world".as_bytes()).await?;
    let overlaid_checksum = hash(&overlaid_file).await?;
    let physical_checksum = hash(&physical_file).await?;

    // compare again
    tracing::info!(
        overlay = overlaid_checksum,
        physical = physical_checksum,
        "Post-write to overlay, compared digests between overlay and physical, they are {}",
        if overlaid_checksum.eq(&physical_checksum) {
            "equal"
        } else {
            "not equal"
        }
    );

    if overlaid_checksum.eq(&physical_checksum) {
        return Err(anyhow!("Files should not be equal after write to overlay!"));
    }

    Ok(())
}

async fn list(root: &VfsPath) -> Result<Vec<VfsPath>> {
    let iter = root.read_dir().await?;

    Ok({
        let mut entries = iter.collect::<Vec<VfsPath>>().await;
        entries.sort_by_key(|p| p.as_str().to_string());
        entries
    })
}

async fn hash(file: &VfsPath) -> Result<String> {
    let contents = read(file).await?;

    Ok(hex::encode(Sha256::digest(&contents).as_slice()))
}

async fn read(file: &VfsPath) -> Result<Vec<u8>> {
    let mut reader = file.open_file().await?;

    let mut contents = Vec::new();
    reader.read_to_end(&mut contents).await?;

    Ok(contents)
}

async fn write(file: &VfsPath, data: &[u8]) -> Result<()> {
    let mut writer = file.create_file().await?;
    writer.write_all(data).await?;
    writer.flush().await?;
    Ok(())
}

fn init_logging() {
    tracing_subscriber::fmt()
        .compact()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::new("debug"))
        .init();
}
