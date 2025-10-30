use cli_helpers::prelude::*;
use image_scraper::{
    client::Client,
    store::{Action, PrefixPartLengths, Store},
};
use image_scraper_index::{Entry, db::Database};
use std::{collections::BTreeMap, path::PathBuf};

mod logs;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    opts.verbose.init_logging()?;

    match opts.command {
        Command::DownloadAll {
            store,
            prefix,
            delay_ms,
        } => {
            let inferred_prefix_part_length = Store::infer_prefix_part_lengths(&store)?;

            let prefix_part_lengths = check_prefix_part_lengths(
                inferred_prefix_part_length,
                prefix.map(|prefix_part_lengths| prefix_part_lengths.0),
            )?;

            let store = Store::new(&store).with_prefix_part_lengths(prefix_part_lengths)?;
            let client = Client::new(store);

            let mut writer = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(std::io::stdout());

            for line in std::io::stdin().lines() {
                let line = line?;

                match client.download(&line).await {
                    Ok(Ok((_, action))) => {
                        match action {
                            Action::Added { entry, image_type } => {
                                writer.write_record([
                                    "A",
                                    &format!("{:x?}", entry.digest),
                                    &image_type.to_string(),
                                    &line,
                                ])?;
                            }
                            Action::Found { entry } => {
                                writer.write_record([
                                    "F",
                                    &format!("{:x?}", entry.digest),
                                    "",
                                    &line,
                                ])?;
                            }
                        }

                        Ok(())
                    }
                    Ok(Err(status_code)) => {
                        writer.write_record(["E", &status_code.as_u16().to_string(), "", ""])?;

                        Ok(())
                    }
                    Err(error) => {
                        writer.flush()?;
                        Err(error)
                    }
                }?;

                if let Some(delay_ms) = delay_ms {
                    tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
                }
            }
        }
        Command::List {
            store,
            prefix,
            validate,
        } => {
            let inferred_prefix_part_length = Store::infer_prefix_part_lengths(&store)?;

            let prefix_part_lengths = check_prefix_part_lengths(
                inferred_prefix_part_length,
                prefix.map(|prefix_part_lengths| prefix_part_lengths.0),
            )?;

            let store = Store::new(&store).with_prefix_part_lengths(prefix_part_lengths)?;

            if validate {
                for entry in store.entries() {
                    let entry = entry?;

                    println!("{}", entry.path.as_os_str().to_string_lossy());
                }
            } else {
                for entry in store.entries().validate_fail_fast() {
                    let entry = entry?;

                    println!("{}", entry.path.as_os_str().to_string_lossy());
                }
            }
        }
        Command::IndexImport { index } => {
            let index = Database::open(&index)?;

            let mut reader = csv::ReaderBuilder::new()
                .has_headers(false)
                .from_reader(std::io::stdin());

            let mut count = 0;
            let mut image_type_map = BTreeMap::new();
            let mut found_leftovers = vec![];

            for result in reader.deserialize::<logs::DownloadLogEntry>() {
                let log_entry = result?;

                match log_entry.status {
                    logs::DownloadStatus::Added => {
                        if let Some(image_type) = log_entry.image_type.value() {
                            image_type_map.insert(log_entry.digest, image_type);

                            index.add(
                                &log_entry.url,
                                Entry {
                                    timestamp: log_entry.timestamp,
                                    digest: md5::Digest(log_entry.digest),
                                    image_type,
                                },
                            )?;

                            count += 1;
                        }
                    }
                    logs::DownloadStatus::Found => match image_type_map.get(&log_entry.digest) {
                        Some(image_type) => {
                            index.add(
                                &log_entry.url,
                                Entry {
                                    timestamp: log_entry.timestamp,
                                    digest: md5::Digest(log_entry.digest),
                                    image_type: *image_type,
                                },
                            )?;

                            count += 1;
                        }
                        None => {
                            found_leftovers.push(log_entry);
                        }
                    },
                }
            }

            let mut final_leftovers = vec![];

            for log_entry in found_leftovers {
                match image_type_map.get(&log_entry.digest) {
                    Some(image_type) => {
                        index.add(
                            &log_entry.url,
                            Entry {
                                timestamp: log_entry.timestamp,
                                digest: md5::Digest(log_entry.digest),
                                image_type: *image_type,
                            },
                        )?;

                        count += 1;
                    }
                    None => {
                        final_leftovers.push(log_entry);
                    }
                }
            }

            log::info!("Added {} entries", count);
            log::warn!("{} leftover found entries", final_leftovers.len())
        }
        Command::IndexDump { index } => {
            let index = Database::open(&index)?;

            for result in index.iter() {
                let (url, result) = result?;

                match result {
                    Ok(entry) => {
                        println!(
                            "S,{},{},{},{:x}",
                            url,
                            entry.timestamp.timestamp(),
                            image_scraper::image_type::ImageType::from(entry.image_type),
                            entry.digest
                        );
                    }
                    Err(timestamp) => {
                        println!("E,{},{},,", url, timestamp.timestamp());
                    }
                }
            }
        }
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("CLI argument reading error")]
    Args(#[from] cli_helpers::Error),
    #[error("CSV error")]
    Csv(#[from] csv::Error),
    #[error("Client error")]
    Client(#[from] image_scraper::client::Error),
    #[error("Store error")]
    Store(#[from] image_scraper::store::Error),
    #[error("Store initialization error")]
    StoreInitialization(#[from] image_scraper::store::InitializationError),
    #[error("Store iteration error")]
    StoreIteration(#[from] image_scraper::store::IterationError),
    #[error("Index database error")]
    IndexDatabase(#[from] image_scraper_index::db::Error),
    #[error("Missing prefix part lengths")]
    MissingPrefixPartLengths,
    #[error("Prefix part lengths mismatch")]
    PrefixPartLengthsMismatch {
        inferred: Vec<usize>,
        provided: Vec<usize>,
    },
}

#[derive(Debug, Parser)]
#[clap(name = "image-scraper", version, author)]
struct Opts {
    #[clap(flatten)]
    verbose: Verbosity,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Parser)]
enum Command {
    /// Download a list of URLs provided on standard input
    DownloadAll {
        #[clap(long)]
        store: PathBuf,
        #[clap(long)]
        prefix: Option<PrefixPartLengths>,
        #[clap(long)]
        delay_ms: Option<u64>,
    },
    /// List the contents of an image store, optionally validating
    List {
        #[clap(long)]
        store: PathBuf,
        #[clap(long)]
        prefix: Option<PrefixPartLengths>,
        #[clap(long)]
        validate: bool,
    },
    IndexImport {
        #[clap(long)]
        index: PathBuf,
    },
    IndexDump {
        #[clap(long)]
        index: PathBuf,
    },
}

fn check_prefix_part_lengths(
    inferred: Option<Vec<usize>>,
    provided: Option<Vec<usize>>,
) -> Result<Vec<usize>, Error> {
    match (inferred, provided) {
        (Some(inferred), Some(provided)) => {
            if inferred == provided {
                Ok(inferred)
            } else {
                Err(Error::PrefixPartLengthsMismatch { inferred, provided })
            }
        }
        (Some(inferred), None) => Ok(inferred),
        (None, Some(provided)) => Ok(provided),
        (None, None) => Err(Error::MissingPrefixPartLengths),
    }
}
