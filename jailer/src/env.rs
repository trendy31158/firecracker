use std::ffi::CString;
use std::fs::{self, canonicalize};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::ArgMatches;
use libc;

use cgroup::Cgroup;
use MAX_ID_LENGTH;
use {into_cstring, Error, Result};

pub struct Env {
    id: String,
    numa_node: u32,
    chroot_dir: PathBuf,
    exec_file_path: PathBuf,
    uid: u32,
    gid: u32,
}

// Function will only allow alphanumeric characters and hyphens.
// Also a MAX_ID_LENGTH restriction is applied.
fn check_id(input: &str) -> Result<()> {
    let no_hyphens = str::replace(input, "-", "");
    for c in no_hyphens.chars() {
        if !c.is_alphanumeric() {
            return Err(Error::InvalidCharId);
        }
    }
    if input.len() > MAX_ID_LENGTH {
        return Err(Error::InvalidLengthId);
    }
    Ok(())
}

impl Env {
    pub fn new(args: ArgMatches) -> Result<Self> {
        // All arguments are either mandatory, or have default values, so the unwraps
        // should not fail.
        let id = args.value_of("id").unwrap();

        // Check that id has only alphanumeric chars and hyphens in it.
        check_id(id)?;

        let numa_node_str = args.value_of("numa_node").unwrap();
        let numa_node = numa_node_str
            .parse::<u32>()
            .map_err(|_| Error::NumaNode(String::from(numa_node_str)))?;

        let exec_file = args.value_of("exec_file").unwrap();
        let exec_file_path =
            canonicalize(exec_file).map_err(|e| Error::Canonicalize(PathBuf::from(exec_file), e))?;

        if !exec_file_path.is_file() {
            return Err(Error::NotAFile(exec_file_path));
        }

        let chroot_base = args.value_of("chroot_base").unwrap();

        let mut chroot_dir = canonicalize(chroot_base)
            .map_err(|e| Error::Canonicalize(PathBuf::from(chroot_base), e))?;

        chroot_dir.push(
            exec_file_path
                .file_name()
                .ok_or_else(|| Error::FileName(exec_file_path.clone()))?,
        );
        chroot_dir.push(id);
        chroot_dir.push("root");

        let uid_str = args.value_of("uid").unwrap();
        let uid = uid_str
            .parse::<u32>()
            .map_err(|_| Error::Uid(String::from(uid_str)))?;

        let gid_str = args.value_of("gid").unwrap();
        let gid = gid_str
            .parse::<u32>()
            .map_err(|_| Error::Gid(String::from(gid_str)))?;

        Ok(Env {
            id: id.to_string(),
            numa_node,
            chroot_dir,
            exec_file_path,
            uid,
            gid,
        })
    }

    pub fn chroot_dir(&self) -> &Path {
        self.chroot_dir.as_path()
    }

    pub fn gid(&self) -> u32 {
        self.gid
    }

    pub fn uid(&self) -> u32 {
        self.uid
    }

    pub fn run(mut self) -> Result<()> {
        // Create the jail folder.
        // TODO: the final part of chroot_dir ("<id>/root") should not exist, if the id is never
        // reused. Is this a reasonable assumption? Should we check for this and return an error?
        // If we choose to do that here, we should extend the same extra functionality to the Cgroup
        // module, where we also create a folder hierarchy which depends on the id.
        fs::create_dir_all(&self.chroot_dir)
            .map_err(|e| Error::CreateDir(self.chroot_dir.clone(), e))?;

        let exec_file_name = self
            .exec_file_path
            .file_name()
            .ok_or_else(|| Error::FileName(self.exec_file_path.clone()))?;

        let chroot_exec_file = PathBuf::from("/").join(exec_file_name);

        // We do a quick push here to get the global path of the executable inside the chroot,
        // without having to create a new PathBuf. We'll then do a pop to revert to the actual
        // chroot_dir right after the copy.
        // TODO: just now wondering ... is doing a push()/pop() thing better than just creating
        // a new PathBuf, with something like chroot_dir.join(exec_file_name) ?!
        self.chroot_dir.push(exec_file_name);

        // TODO: hard link instead of copy? This would save up disk space, but hard linking is
        // not always possible :(
        fs::copy(&self.exec_file_path, &self.chroot_dir)
            .map_err(|e| Error::Copy(self.exec_file_path.clone(), self.chroot_dir.clone(), e))?;

        // Pop exec_file_name.
        self.chroot_dir.pop();

        // We have to setup cgroups at this point, because we can't do it anymore after chrooting.
        let cgroup = Cgroup::new(self.id.as_str(), self.numa_node, exec_file_name)?;
        cgroup.attach_pid()?;

        let chroot_dir: CString = into_cstring(self.chroot_dir)?;
        let ret = unsafe { libc::chroot(chroot_dir.as_ptr()) };
        if ret < 0 {
            return Err(Error::Chroot(ret));
        }

        Err(Error::Exec(
            Command::new(chroot_exec_file)
                .arg("--jailed")
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .uid(self.uid)
                .gid(self.gid)
                .exec(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap_app;

    fn make_args<'a>(
        node: &str,
        id: &str,
        exec_file: &str,
        uid: &str,
        gid: &str,
        chroot_base: &str,
    ) -> ArgMatches<'a> {
        let app = clap_app();

        let arg_vec = vec![
            "jailer",
            "--node",
            node,
            "--id",
            id,
            "--exec-file",
            exec_file,
            "--uid",
            uid,
            "--gid",
            gid,
            "--chroot-base-dir",
            chroot_base,
        ];

        app.get_matches_from_safe(arg_vec).unwrap()
    }

    #[test]
    fn test_new_env() {
        let node = "1";
        let id = "bd65600d-8669-4903-8a14-af88203add38";
        let exec_file = "/proc/cpuinfo";
        let uid = "1001";
        let gid = "1002";
        let chroot_base = "/";

        // This should be fine.
        let good_env = Env::new(make_args(node, id, exec_file, uid, gid, chroot_base))
            .expect("This new environment should be created successfully.");

        let mut chroot_dir = PathBuf::from(chroot_base);
        chroot_dir.push(Path::new(exec_file).file_name().unwrap());
        chroot_dir.push(id);
        chroot_dir.push("root");

        assert_eq!(good_env.chroot_dir(), chroot_dir);
        assert_eq!(format!("{}", good_env.gid()), gid);
        assert_eq!(format!("{}", good_env.uid()), uid);

        // Not fine - invalid node.
        assert!(Env::new(make_args("zzz", id, exec_file, uid, gid, chroot_base)).is_err());

        // Not fine - invalid id.
        assert!(
            Env::new(make_args(
                node,
                "/ad./sa12",
                exec_file,
                uid,
                gid,
                chroot_base
            )).is_err()
        );

        // Not fine - inexistent (hopefully) exec_file.
        assert!(
            Env::new(make_args(
                node,
                id,
                "/this!/file!/should!/not!/exist!/",
                uid,
                gid,
                chroot_base
            )).is_err()
        );

        // Not fine - invalid uid.
        assert!(Env::new(make_args(node, id, exec_file, "zzz", gid, chroot_base)).is_err());

        // Not fine - invalid gid.
        assert!(Env::new(make_args(node, id, exec_file, uid, "zzz", chroot_base)).is_err());

        // The chroot-base-dir param is not validated by Env::new, but rather in run, when we
        // actually attempt to create the folder structure.
    }

    #[test]
    fn test_check_id() {
        assert!(check_id("12-3aa").is_ok());
        assert!(check_id("12:3aa").is_err());
        assert!(check_id("①").is_err());
        let mut long_str = "".to_string();
        for _n in 1..=MAX_ID_LENGTH + 1 {
            long_str.push('a');
        }
        assert!(check_id(long_str.as_str()).is_err());
    }
}
