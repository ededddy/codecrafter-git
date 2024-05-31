use std::path::PathBuf;

use clap::Subcommand;

pub(crate) mod cat_file;
pub(crate) mod commit;
pub(crate) mod commit_tree;
pub(crate) mod hash_object;
pub(crate) mod ls_tree;
pub(crate) mod write_tree;

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,
        object_hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file: PathBuf,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,
        tree_hash: String,
    },
    WriteTree,
    CommitTree {
        #[clap(short = 'm')]
        message: String,

        #[clap(short = 'p')]
        parent_hash: Option<String>,

        tree_hash: String,
    },
    Commit {
        #[clap(short = 'm')]
        message: String,
    },
}
