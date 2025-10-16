use cli_helpers::prelude::*;
use image_scraper::{
    client::Client,
    store::{Action, Store},
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let opts: Opts = Opts::parse();
    opts.verbose.init_logging()?;

    match opts.command {
        Command::DownloadAll { store, prefix } => {
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
                    Ok(Ok(action)) => {
                        match action {
                            Action::Added { entry, image_type } => {
                                writer.write_record([
                                    "A",
                                    &format!("{:x?}", entry.digest),
                                    &image_type
                                        .map(|image_type| {
                                            format!("{:?}", image_type).to_ascii_lowercase()
                                        })
                                        .unwrap_or_default(),
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

#[derive(Clone, Debug)]
struct PrefixPartLengths(Vec<usize>);

impl std::str::FromStr for PrefixPartLengths {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split('/')
            .map(|prefix_part_length| prefix_part_length.parse())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| s.to_string())
            .map(PrefixPartLengths)
    }
}
