use std::{
    ffi::OsStr,
    fmt::Debug,
    fs, io,
    path::{Path, PathBuf},
};

use crate::{cfg::SshCfg, format_rverr};
use crate::{
    result::{to_rv, RvResult},
    tools_data::BboxExportData,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

lazy_static! {
    pub static ref DEFAULT_TMPDIR: PathBuf = std::env::temp_dir().join("rvimage");
}
lazy_static! {
    pub static ref DEFAULT_HOMEDIR: PathBuf = match dirs::home_dir() {
        Some(p) => p.join(".rvimage"),
        _ => std::env::temp_dir().join("rvimage"),
    };
}

pub fn read_to_string<P>(p: P) -> RvResult<String>
where
    P: AsRef<Path> + Debug,
{
    fs::read_to_string(&p).map_err(|e| format_rverr!("could not read {:?} due to {:?}", p, e))
}
pub trait PixelEffect: FnMut(u32, u32) {}
impl<T: FnMut(u32, u32)> PixelEffect for T {}
pub fn filename_in_tmpdir(path: &str, tmpdir: &str) -> RvResult<String> {
    let path = PathBuf::from_str(path).unwrap();
    let fname = osstr_to_str(path.file_name()).map_err(to_rv)?;
    Path::new(tmpdir)
        .join(fname)
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| format_rverr!("could not transform {:?} to &str", fname))
}

pub fn path_to_str(p: &Path) -> RvResult<&str> {
    osstr_to_str(Some(p.as_os_str()))
        .map_err(|e| format_rverr!("could not transform '{:?}' due to '{:?}'", p, e))
}
pub fn osstr_to_str(p: Option<&OsStr>) -> io::Result<&str> {
    p.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("{:?} not found", p)))?
        .to_str()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{:?} not convertible to unicode", p),
            )
        })
}

pub fn to_stem_str(p: &Path) -> RvResult<&str> {
    osstr_to_str(p.file_stem())
        .map_err(|e| format_rverr!("could not transform '{:?}' due to '{:?}'", p, e))
}

pub fn to_name_str(p: &Path) -> RvResult<&str> {
    osstr_to_str(p.file_name())
        .map_err(|e| format_rverr!("could not transform '{:?}' due to '{:?}'", p, e))
}
#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq)]
pub enum ConnectionData {
    Ssh(SshCfg),
    #[default]
    None,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct MetaData {
    pub file_path: Option<String>,
    pub connection_data: ConnectionData,
    pub opened_folder: Option<String>,
    pub export_folder: Option<String>,
}
impl MetaData {
    pub fn from_filepath(file_path: String) -> Self {
        MetaData {
            file_path: Some(file_path),
            connection_data: ConnectionData::None,
            opened_folder: None,
            export_folder: None,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ExportData {
    pub opened_folder: String,
    pub connection_data: ConnectionData,
    pub bbox_data: Option<BboxExportData>,
}

pub struct Defer<F: FnMut()> {
    pub func: F,
}
impl<F: FnMut()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.func)();
    }
}
#[macro_export]
macro_rules! defer {
    ($f:expr) => {
        let _dfr = $crate::file_util::Defer { func: $f };
    };
}
pub fn checked_remove<'a, P: AsRef<Path> + Debug>(
    path: &'a P,
    func: fn(p: &'a P) -> io::Result<()>,
) {
    match func(path) {
        Ok(_) => println!("removed {:?}", path),
        Err(e) => println!("could not remove {:?} due to {:?}", path, e),
    }
}
#[macro_export]
macro_rules! defer_folder_removal {
    ($path:expr) => {
        let func = || $crate::file_util::checked_remove($path, std::fs::remove_dir_all);
        $crate::defer!(func);
    };
}
#[macro_export]
macro_rules! defer_file_removal {
    ($path:expr) => {
        let func = || $crate::file_util::checked_remove($path, std::fs::remove_file);
        $crate::defer!(func);
    };
}

#[allow(clippy::needless_lifetimes)]
pub fn files_in_folder<'a, P>(
    folder: P,
    extension: &'a str,
) -> io::Result<impl Iterator<Item = PathBuf> + 'a>
where
    P: AsRef<Path>,
{
    Ok(fs::read_dir(folder)?
        .flatten()
        .map(|de| de.path())
        .filter(|p| p.is_file() && (p.extension() == Some(OsStr::new(extension)))))
}

pub fn write<P, C>(path: P, contents: C) -> RvResult<()>
where
    P: AsRef<Path> + Debug,
    C: AsRef<[u8]>,
{
    fs::write(&path, contents)
        .map_err(|e| format_rverr!("could not write to {:?} since {:?}", path, e))
}

#[macro_export]
macro_rules! p_to_rv {
    ($path:expr, $expr:expr) => {
        $expr.map_err(|e| format_rverr!("{:?}, failed on {:?}", e, $path))
    };
}
