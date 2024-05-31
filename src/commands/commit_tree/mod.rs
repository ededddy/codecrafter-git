use crate::objects::{Kind, Object};
use anyhow::Context;
use chrono::Local;
use std::env;
use std::fmt::Write;
use std::io::Cursor;

pub(crate) fn write_commit(
    message: &str,
    tree_hash: &str,
    parent_hash: Option<&str>,
) -> anyhow::Result<[u8; 20]> {
    let mut commit = String::new();
    writeln!(commit, "tree {tree_hash}")?;
    if let Some(parent_hash) = parent_hash {
        writeln!(commit, "parent {parent_hash}")?;
    }
    let (name, email) =
        if let (Some(name), Some(email)) = (env::var_os("NAME"), env::var_os("EMAIL")) {
            let name = name
                .into_string()
                .map_err(|_| anyhow::anyhow!("$NAME is invalid utf-8"))?;
            let email = email
                .into_string()
                .map_err(|_| anyhow::anyhow!("$EMAIL is invalid utf-8"))?;
            (name, email)
        } else {
            (
                String::from("Eddy Lei"),
                String::from("eddylei070300@gmail.com"),
            )
        };

    let time = Local::now();
    let timezone = (Local::now().offset().local_minus_utc() / 3600).to_string();
    let time = time.timestamp().to_string();
    writeln!(commit, "author {name} <{email}> {time} +{:0^4}", timezone)?;
    writeln!(
        commit,
        "committer {name} <{email}> {time} +{:0^4}",
        timezone
    )?;
    writeln!(commit)?;
    writeln!(commit, "{message}")?;

    Object {
        kind: Kind::Commit,
        expected_size: commit.len() as u64,
        reader: Cursor::new(commit),
    }
    .write_to_objects()
    .context("stream tree object into tree object file")
}

pub(crate) fn invoke(
    message: String,
    tree_hash: String,
    parent_hash: Option<String>,
) -> anyhow::Result<()> {
    // NOTE: ? at writeln so will never trigger warning as it returns Result
    let hash =
        write_commit(&message, &tree_hash, parent_hash.as_deref()).context("create commit")?;
    println!("{}", hex::encode(hash));
    Ok(())
}
