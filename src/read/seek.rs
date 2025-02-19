// Copyright (c) 2022 Harry [Majored] [hello@majored.pw]
// MIT License (https://github.com/Majored/rs-async-zip/blob/main/LICENSE)

//! A ZIP reader which acts over a seekable source.
//!
//! ### Example
//! ```no_run
//! # use async_zip::read::seek::ZipFileReader;
//! # use async_zip::error::Result;
//! # use tokio::io::AsyncReadExt;
//! # use tokio::fs::File;
//! #
//! async fn run() -> Result<()> {
//!     let mut data = File::open("./foo.zip").await?;
//!     let mut reader = ZipFileReader::new(&mut data).await?;
//!
//!     let mut data = Vec::new();
//!     let mut entry = reader.entry(0).await?;
//!     entry.read_to_end(&mut data).await?;
//!
//!     // Use data within current scope.
//!
//!     Ok(())
//! }
//! ```

use crate::error::{Result, ZipError};
use crate::file::ZipFile;
pub use crate::read::io::entry::ZipEntryReader;

use tokio::io::{AsyncRead, AsyncSeek, BufReader};

/// A ZIP reader which acts over a seekable source.
#[derive(Clone)]
pub struct ZipFileReader<R> {
    reader: R,
    file: ZipFile,
}

impl<R> ZipFileReader<R>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    /// Constructs a new ZIP reader from a seekable source.
    pub async fn new(mut reader: R) -> Result<ZipFileReader<R>> {
        let file = crate::read::file(&mut reader).await?;
        Ok(ZipFileReader { reader, file })
    }

    /// Constructs a ZIP reader from a seekable source and zip file information
    /// derived from that source.
    ///
    /// This can be used to avoid parsing the same zip file several times
    ///
    /// ```no_run
    /// # use tokio::fs::File;
    /// # use async_zip::read::seek::ZipFileReader;
    /// # use async_zip::error::Result;
    /// #
    /// # async fn run() -> Result<()> {
    /// let reader1 = File::open("/somefile").await?;
    /// let reader2 = File::open("/somefile").await?;
    ///
    /// let zfr1 = ZipFileReader::new(reader1).await?;
    ///
    /// // Avoid parsing the zip file again
    /// let zfr2 = ZipFileReader::from_parts(reader2, zfr1.file().clone());
    /// # Ok(())
    /// # }
    /// ```
    ///
    pub fn from_parts(reader: R, file: ZipFile) -> ZipFileReader<R> {
        ZipFileReader { reader, file }
    }

    /// Returns this ZIP file's information.
    pub fn file(&self) -> &ZipFile {
        &self.file
    }

    /// Returns a mutable reference to the inner reader
    pub fn inner_mut(&mut self) -> &mut R {
        &mut self.reader
    }

    /// Unwraps this `ZipFileReader<R>`, returning the underlying reader.
    pub fn into_inner(self) -> R {
        self.reader
    }

    /// Returns a new entry reader if the provided index is valid.
    pub async fn entry(&mut self, index: usize) -> Result<ZipEntryReader<'_, R>> {
        let stored_entry = self.file.entries.get(index).ok_or(ZipError::EntryIndexOutOfBounds)?;

        let mut reader = BufReader::new(&mut self.reader);

        stored_entry.seek_to_data_offset(&mut reader).await?;

        Ok(ZipEntryReader::new_with_borrow(
            reader,
            stored_entry.entry.compression(),
            stored_entry.entry.uncompressed_size().into(),
        ))
    }

    /// Returns a new entry reader if the provided index is valid.
    /// Consumes self
    pub async fn into_entry<'a>(self, index: usize) -> Result<ZipEntryReader<'a, R>>
    where
        R: 'a,
    {
        let stored_entry = self.file.entries.get(index).ok_or(ZipError::EntryIndexOutOfBounds)?;

        let mut reader = BufReader::new(self.reader);

        stored_entry.seek_to_data_offset(&mut reader).await?;

        Ok(ZipEntryReader::new_with_owned(
            reader,
            stored_entry.entry.compression(),
            stored_entry.entry.uncompressed_size().into(),
        ))
    }
}
