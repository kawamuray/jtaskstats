use clap::{App, Arg};
use linux_taskstats::cmd;
use linux_taskstats::format::HeaderFormat;
use std::collections::HashMap;
use std::env;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{self, Command, Stdio};
use regex::Regex;
use tempfile::{self, NamedTempFile};

const JTHREAD_INFO_HEADER: &str = "Thread ";

fn main() {
    let matches = App::new("A command line interface to Linux taskstats for JVM")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Show all available stats for each tasks in verbose style output"),
        )
        .arg(
            Arg::with_name("show-delays")
                .short("d")
                .long("delay")
                .help("Show only delay accounting related stats"),
        )
        .arg(
            Arg::with_name("full-name")
                .long("full-name")
                .help("Show full (not-shortend) thread name"),
        )
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
        header_format: JavaHeaderFormat {
            full_name: matches.is_present("full-name"),
            mapping: &mapping,
        },
    };
    cmd::taskstats_main(config);
}

struct ThreadInfo {
    tid: i64,
    name: String,
    is_java: bool,
}

enum JdkVersion {
    Jdk9OrHigher,
    Jdk8OrLower,
}

impl ThreadInfo {
    const MAX_SHORT_NAME_LEN: usize = 50;

    fn short_name(&self) -> String {
        if self.name.len() > Self::MAX_SHORT_NAME_LEN {
            let half = Self::MAX_SHORT_NAME_LEN / 2;
            format!(
                "{}...{}",
                &self.name[..(half - "...".len())],
                &self.name[(self.name.len() - half)..]
            )
        } else {
            self.name.clone()
        }
    }
}

struct JavaHeaderFormat<'a> {
    full_name: bool,
    mapping: &'a HashMap<u32, ThreadInfo>,
}

impl<'a> HeaderFormat for JavaHeaderFormat<'a> {
    fn format(&self, tid: u32) -> String {
        if let Some(info) = self.mapping.get(&tid) {
            let short: String;
            let name = if self.full_name {
                &info.name
            } else {
                short = info.short_name();
                &short
            };
            format!("{} Thread {} - {}", tid, info.tid, name)
        } else {
            format!("{} Thread UNKNOWN", tid)
        }
    }
}

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
    file.write_all(include_bytes!("../jthreadinfo/build/libs/jthreadinfo.jar"))
        .expect("write jthreadinfo jar");
    file
}

fn get_jvm_threads(pid: u32) -> Result<HashMap<u32, ThreadInfo>, io::Error> {
    let jthreadinfo_jar = prepare_jthreadinfo_jar();
    let mut child = Command::new(format!("{}/bin/java", java_home()));

    match jdk_version() {
        JdkVersion::Jdk9OrHigher => {
            child.args(&[
                "-cp",
                jthreadinfo_jar.as_ref().to_str().unwrap(),
                "--add-modules", "jdk.hotspot.agent",
                "--add-exports", "jdk.hotspot.agent/sun.jvm.hotspot.tools=ALL-UNNAMED",
                "--add-exports", "jdk.hotspot.agent/sun.jvm.hotspot.runtime=ALL-UNNAMED",
                "--add-exports", "jdk.hotspot.agent/sun.jvm.hotspot.oops=ALL-UNNAMED"
            ]);
        }
        JdkVersion::Jdk8OrLower => {
            child.args(&[
                "-cp",
                &format!(
                    "{}:{}",
                    jthreadinfo_jar.as_ref().to_str().unwrap(),
                    jdi_jar_path().to_str().unwrap(),
                ),
            ]);
        }
    }
    let mut child = child
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
            [_, nid, tid, name] => {
                let nid: u32 = nid.parse().expect("parse nid");
                let tid: i64 = tid.parse().expect("parse tid");
                let name = (*name).to_owned();
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

fn jdk_version() -> JdkVersion {
    let output = Command::new(format!("{}/bin/java", java_home()))
        .arg("-version")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .expect("get java version")
        .stderr;
    let output = std::str::from_utf8(&output).unwrap();
    let regex = Regex::new("version \"(.+)\"").unwrap();

    let version_string = &regex
        .captures(output)
        .expect("parse output")[1];

    if version_string.starts_with("1.") {
        JdkVersion::Jdk8OrLower
    } else {
        JdkVersion::Jdk9OrHigher
    }
}
