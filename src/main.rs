mod storage;
mod uri;

use anyhow::Result;
use clap::{ArgGroup, Parser};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    env,
    path::{Path, PathBuf},
    process::{exit, Command},
    sync::Arc,
};
use storage::{CloudStorage, GCSStorage, S3Storage};
use tempfile::NamedTempFile;
use tokio::{sync::mpsc, task};

/// Cloud Storage Editor Utility
#[derive(Parser, Debug)]
#[command(
    name = "eo",
    version = "1.0",
    about = "A tool to edit files directly in cloud object storage"
)]
#[command(group(ArgGroup::new("object_config")
    .required(true)
    .args(&["uri", "bucket"])
))]
struct Cli {
    /// Cloud storage provider (s3 for AWS S3, gcs for Google Cloud Storage)
    #[arg(short, long, default_value = "s3")]
    storage: String,

    /// Bucket name (mutually exclusive with --uri)
    #[arg(long, short)]
    bucket: Option<String>,

    /// Object key (mutually exclusive with --uri)
    #[arg(long, short, requires = "bucket")]
    key: Option<String>,

    /// Object URL (optional, mutually exclusive with --bucket and --key)
    #[arg(long, short)]
    uri: Option<String>,

    /// Cloud region (optional, defaults to environment config)
    #[arg(long, short)]
    region: Option<String>,

    /// Local file path (optional, if you want to use your own temp file location)
    #[arg(long, short)]
    file_path: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Parse the URI or use the bucket and key provided.
    let (bucket, key) =
        uri::parse_uri(&cli.uri)?.unwrap_or_else(|| (cli.bucket.unwrap(), cli.key.unwrap()));

    let storage_client = match cli.storage.as_str() {
        "s3" => CloudStorage::S3(S3Storage::new(cli.region).await),
        "gcs" => CloudStorage::GCS(GCSStorage::new(cli.region).await),
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported storage provider: {}",
                cli.storage
            ))
        }
    };

    cloud_edit(storage_client, &bucket, &key, cli.file_path).await?;

    Ok(())
}

async fn cloud_edit(
    client: CloudStorage,
    bucket: &str,
    key: &str,
    file_path: Option<String>,
) -> Result<()> {
    let client = Arc::new(client);

    // Create or use a local temporary file for editing
    let temp_path = Arc::new(
        file_path
            .map(PathBuf::from)
            .unwrap_or_else(|| NamedTempFile::new().unwrap().into_temp_path().to_path_buf()),
    );

    // Channel to signal file watcher termination
    let (stop_tx, stop_rx) = mpsc::channel(1);

    // Download file from cloud storage to temporary location
    client.download_file(bucket, key, &temp_path).await?;

    // Watch file changes and sync with cloud storage
    let file_watcher_handle = task::spawn(watch_and_sync_file(
        Arc::clone(&temp_path),
        Arc::clone(&client),
        bucket.to_string(),
        key.to_string(),
        stop_rx,
    ));

    // Open the file in the user's preferred editor
    edit_file(&temp_path)?;

    // Signal the watcher to stop after the editor closes
    stop_tx.send(()).await?;

    // Wait for the file watcher to finish
    file_watcher_handle.await??;

    Ok(())
}

fn edit_file(file_path: &Path) -> Result<()> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());
    let status = Command::new(editor).arg(file_path).status()?;

    if !status.success() {
        eprintln!("Editor process exited with non-zero status.");
        exit(1);
    }

    Ok(())
}

async fn watch_and_sync_file(
    file_path: Arc<PathBuf>,
    storage_client: Arc<CloudStorage>,
    bucket: String,
    key: String,
    mut stop_rx: mpsc::Receiver<()>,
) -> Result<()> {
    // Note: we don't debounce the file modification events here, so all saves will trigger an upload.
    // Since the uploads are async, there's no guarantee they are processed in order by the storage system.
    let (tx, mut rx) = mpsc::unbounded_channel();

    // TODO: debounce? See multiple uploads per save.
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(Event {
                kind: EventKind::Modify(_),
                ..
            }) = res
            {
                let _ = tx.send(());
            }
        },
        notify::Config::default(),
    )?;
    watcher.watch(&file_path, RecursiveMode::NonRecursive)?;

    loop {
        tokio::select! {
            // Check for file modification
            Some(_) = rx.recv() => {
                storage_client.upload_file(&bucket, &key, &file_path).await?;
            }
            // Check if we received the stop signal
            _ = stop_rx.recv() => {
                break;
            }
        }
    }

    Ok(())
}
