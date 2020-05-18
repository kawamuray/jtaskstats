use clap::{App, Arg};
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use taskstats::cmd;
use taskstats::format::HeaderFormat;
use tempfile::{self, NamedTempFile};

const JTHREAD_INFO_HEADER: &str = "Thread ";

fn error_exit(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    process::exit(1);
}

fn java_home() -> String {
    if let Ok(java_home) = env::var("JAVA_HOME") {
        java_home
    } else {
        error_exit("JAVA_HOME is not set");
    }
}

fn jdi_jar_path() -> PathBuf {
    let path = PathBuf::from(format!("{}/lib/sa-jdi.jar", java_home()));
    if !path.exists() {
        error_exit(&format!(
            "required sa-jdi.jar could not found in {:?}",
            path
        ));
    }
    path
}

fn prepare_jthreadinfo_jar() -> NamedTempFile {
    let mut file = tempfile::Builder::new()
        .prefix("jtaskstats-jthreadinfo")
        .suffix(".jar")
        .rand_bytes(6)
        .tempfile()
        .expect("create tempfile for extracting jar");
    file.write(include_bytes!("../jthreadinfo/build/libs/jthreadinfo.jar"))
        .expect("write jthreadinfo jar");
    file
}

fn get_jvm_threads(pid: u32) -> Result<HashMap<u32, ThreadInfo>, io::Error> {
    let jthreadinfo_jar = prepare_jthreadinfo_jar();
    let mut child = Command::new(format!("{}/bin/java", java_home()))
        .args(&[
            "-cp",
            &format!(
                "{}:{}",
                jthreadinfo_jar.as_ref().to_str().unwrap(),
                jdi_jar_path().to_str().unwrap()
            ),
        ])
        .arg("jthreadinfo.JThreadInfo")
        .arg(pid.to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;
    let reader = BufReader::new(child.stdout.take().unwrap());

    let mut mapping = HashMap::new();
    for line in reader.lines() {
        let line = line?;
        if !line.starts_with(JTHREAD_INFO_HEADER) {
            continue;
        }
        match line.splitn(4, ' ').collect::<Vec<_>>().as_slice() {
            &[_, nid, tid, name] => {
                let nid: u32 = nid.parse().expect("parse nid");
                let tid: i64 = tid.parse().expect("parse tid");
                let name = name.to_owned();
                mapping.insert(
                    nid,
                    ThreadInfo {
                        tid,
                        name,
                        is_java: tid != 0,
                    },
                );
            }
            _ => panic!("corrupted line from jthreadinfo: {}", line),
        }
    }

    let status = child.wait()?;
    if !status.success() {
        error_exit("jthreadinfo exit with failure");
    }

    Ok(mapping)
}

fn main() {
    let matches = App::new("A command line interface to Linux taskstats for JVM")
        .arg(Arg::with_name("verbose").short("v").long("verbose"))
        .arg(Arg::with_name("show-delays").short("d").long("delay"))
        .arg(Arg::with_name("JVM_PID").index(1).required(true))
        .get_matches();

    let pid = matches.value_of("JVM_PID").unwrap();
    let pid = match pid.parse::<u32>() {
        Ok(pid) => pid,
        Err(_) => {
            eprintln!("Invalid PID: {}", pid);
            process::exit(1);
        }
    };
    let mapping = match get_jvm_threads(pid) {
        Ok(mapping) => mapping,
        Err(e) => error_exit(&format!(
            "failed to get threads info from target JVM: {}",
            e
        )),
    };

    let mut tids: Vec<_> = mapping
        .iter()
        .filter_map(|(k, v)| if v.is_java { Some(*k) } else { None })
        .collect();
    tids.sort_unstable();
    let config = cmd::Config {
        tids,
        verbose: matches.is_present("verbose"),
        show_delays: matches.is_present("show-delays"),
        header_format: JavaHeaderFormat { mapping: &mapping },
    };
    cmd::taskstats_main(config);
}

struct ThreadInfo {
    tid: i64,
    name: String,
    is_java: bool,
}

struct JavaHeaderFormat<'a> {
    mapping: &'a HashMap<u32, ThreadInfo>,
}

impl<'a> HeaderFormat for JavaHeaderFormat<'a> {
    fn format(&self, tid: u32) -> String {
        if let Some(info) = self.mapping.get(&tid) {
            format!("{} Thread {} - {}", tid, info.tid, info.name)
        } else {
            format!("{} Thread UNKNOWN", tid)
        }
    }
}
