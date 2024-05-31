use std::fs;
use std::path::Path;

use anyhow::Context;
use clap::command;
use clap::Parser;
use commands::Command;

pub(crate) mod commands;
pub(crate) mod objects;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> anyhow::Result<()> {
    // Uncomment this block to pass the first stage
    let args = Args::parse();
    eprintln!("Logs from your program will appear here!");

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory");
        }
        Command::CatFile {
            pretty_print,
            object_hash,
        } => commands::cat_file::invoke(pretty_print, &object_hash)?,
        Command::HashObject { write, file } => commands::hash_object::invoke(write, file)?,
        Command::LsTree {
            name_only,
            tree_hash,
        } => commands::ls_tree::invoke(name_only, &tree_hash)?,
        Command::WriteTree => commands::write_tree::invoke()?,
        Command::CommitTree {
            message,
            parent_hash,
            tree_hash,
        } => commands::commit_tree::invoke(message, tree_hash, parent_hash)?,
        Command::Commit { message } => {
            let head_ref = fs::read_to_string(".git/HEAD").context("read HEAD")?;
            let Some(head_ref) = head_ref.strip_prefix("ref: ") else {
                anyhow::bail!("refusing to commit onto detached HEAD");
            };
            let head_ref = head_ref.trim();
            let parent_hash = fs::read_to_string(format!(".git/{head_ref}"))
                .with_context(|| "read HEAD reference target '{head_ref}'")?;

            let Some(tree_hash) =
                commands::write_tree::write_tree_for(Path::new(".")).context("write tree")?
            else {
                eprintln!("not commiting empty tree");
                return Ok(());
            };

            let commit_hash = commands::commit_tree::write_commit(
                &message,
                &hex::encode(tree_hash),
                Some(&parent_hash),
            )
            .context("write commit")?;

            let commit_hash = hex::encode(commit_hash);

            fs::write(format!(".git/{head_ref}"), &commit_hash)
                .with_context(|| "write commit to HEAD ref")?;

            println!("HEAD is now at {commit_hash}");
        }
    }
    Ok(())
}
