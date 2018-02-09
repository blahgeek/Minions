/*
* @Author: BlahGeek
* @Date:   2017-08-19
* @Last Modified by:   BlahGeek
* @Last Modified time: 2018-02-08
*/

use std::process::{Command, Stdio};

#[derive(Debug)]
pub enum Requirement {
    Executable(String),
    Python3Package(String),
    Python2Package(String),
}

impl Requirement {
    pub fn new(text: &str) -> Option<Requirement> {
        let mut parts: Vec<&str> = text.splitn(2, ":").collect();
        if parts.len() < 2 {
            warn!("Invalid requirement {}", text);
            return None;
        }

        let arg = parts.pop().unwrap();
        let name = parts.pop().unwrap();

        match name {
            "exe" => Some(Requirement::Executable(arg.into())),
            "py3" => Some(Requirement::Python3Package(arg.into())),
            "py2" => Some(Requirement::Python2Package(arg.into())),
            _ => None,
        }
    }

    fn check_executable(exe: &str) -> bool {
        match Command::new("which").arg(exe).stdout(Stdio::null()).status() {
            Err(error) => {
                warn!("Error running which: {}", error);
                false
            },
            Ok(status) => {
                status.success()
            },
        }
    }

    fn check_python_package(pkg: &str, python: &str) -> bool {
        let mut cmd = Command::new(python);
        cmd.arg("-c")
           .arg("import pkg_resources as p;import sys;p.require(sys.argv[1])")
           .arg(pkg);
        match cmd.stdout(Stdio::null()).stderr(Stdio::null()).status() {
            Err(error) => {
                warn!("Error checking python package {} for {}: {}", pkg, python, error);
                false
            },
            Ok(status) => {
                status.success()
            },
        }
    }

    pub fn check(&self) -> bool {
        let ret = match self {
            &Requirement::Executable(ref name) =>
                Requirement::check_executable(&name),
            &Requirement::Python2Package(ref name) =>
                Requirement::check_python_package(&name, "python2"),
            &Requirement::Python3Package(ref name) =>
                Requirement::check_python_package(&name, "python3"),
        };
        trace!("Requirement {:?} check result: {}", &self, ret);
        ret
    }
}
