use anyhow::Context;
use flate2::Compression;
use sha1::Digest;
use sha1::Sha1;
use std::ffi::CStr;
use std::fmt::Display;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::Path;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Kind {
    Blob,
    Tree,
    Commit,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Kind::Blob => write!(f, "blob"),
            Kind::Tree => write!(f, "tree"),
            Kind::Commit => write!(f, "commit"),
        }
    }
}

pub(crate) struct Object<R> {
    pub kind: Kind,
    pub expected_size: u64,
    pub reader: R,
}

impl Object<()> {
    pub(crate) fn blob_from_file(file: impl AsRef<Path>) -> anyhow::Result<Object<impl Read>> {
        let file = file.as_ref();
        let stat = fs::metadata(file).with_context(|| format!("stat : {}", file.display()))?;

        // TODO: race condition if changes happen inbetween file and buffer stream in
        let file = std::fs::File::open(file).with_context(|| format!("open {}", file.display()))?;

        Ok(Object {
            kind: Kind::Blob,
            expected_size: stat.len(),
            reader: file,
        })
    }

    pub(crate) fn read(hash: &str) -> anyhow::Result<Object<impl BufRead>> {
        let object_file = fs::File::open(format!("./.git/objects/{}/{}", &hash[..2], &hash[2..]))
            .context("open in .git/objects");

        match object_file {
            Ok(file_bytes) => {
                let decoder = flate2::read::ZlibDecoder::new(file_bytes);
                let mut decoder = BufReader::new(decoder);
                let mut buf = Vec::new();

                decoder
                    .read_until(0, &mut buf)
                    .context("read header from .git/objects")?;

                let header = CStr::from_bytes_with_nul(&buf)
                    .expect("know there is exactly one null, and it's the end");

                let header = header
                    .to_str()
                    .context(".git/objects file header isn't valid utf-8")?;

                let Some((kind, size)) = header.split_once(' ') else {
                    anyhow::bail!(
                        ".git/objects file header did not start with a known type :'{header}'"
                    );
                };

                let kind = match kind {
                    "blob" => Kind::Blob,
                    "tree" => Kind::Tree,
                    "commit" => Kind::Commit,
                    _ => anyhow::bail!("we do not yet know how to parse a '{kind}'"),
                };

                let size = size
                    .parse::<u64>()
                    .context(".git/objects file header has invalid size : '{size}'")?;

                let decoder = decoder.take(size);
                Ok(Object {
                    kind,
                    expected_size: size,
                    reader: decoder,
                })
            }
            Err(error) => {
                println!("{}", error);
                Err(error)
            }
        }
    }
}

impl<R> Object<R>
where
    R: Read,
{
    pub(crate) fn write(mut self, writer: impl Write) -> anyhow::Result<[u8; 20]> {
        // TODO: race condition if changes happen inbetween file and buffer stream in
        let mut writer = HashWriter {
            writer: flate2::write::ZlibEncoder::new(writer, Compression::default()),
            hasher: Sha1::new(),
        };
        write!(writer, "{} {}\0", self.kind, self.expected_size)?;

        std::io::copy(&mut self.reader, &mut writer).context("stream file into blob")?;
        let _ = writer.writer.finish()?;
        let hash = writer.hasher.finalize();

        Ok(hash.into())
    }

    pub(crate) fn write_to_objects(self) -> anyhow::Result<[u8; 20]> {
        let tmp = "temporary";
        let hash = self
            .write(std::fs::File::create(tmp).context("construct temporary file for blob")?)
            .context("stream file into blob")?;
        let hash_hex = hex::encode(hash);

        let dir_name = format!("./.git/objects/{}", &hash_hex[..2]);
        if !Path::new(&dir_name).exists() {
            fs::create_dir(&dir_name).context("create sub folder under .git/objects")?;
        }

        fs::rename(
            tmp,
            format!("./.git/objects/{}/{}", &hash_hex[..2], &hash_hex[2..]),
        )
        .context("move object blob to .git/objects")?;

        Ok(hash)
    }
}

pub(crate) struct HashWriter<W>
where
    W: Write,
{
    pub(crate) writer: W,
    pub(crate) hasher: Sha1,
}

impl<W> Write for HashWriter<W>
where
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let n = self.writer.write(buf)?;
        self.hasher.update(&buf[..n]);
        Ok(n)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
