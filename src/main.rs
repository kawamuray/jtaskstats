use clap::{App, Arg};
use jni::objects::{JObject, JValue};
use jni::sys::{jlong, jobject, jobjectArray};
use jni::{InitArgsBuilder, JNIEnv, JavaVM};
use std::collections::HashMap;
use std::env;
use std::io;
use std::path::PathBuf;
use std::process;
use taskstats::cmd;
use taskstats::format::{HeaderFormat, Printer};
use taskstats::Client;

fn error_exit(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    process::exit(1);
}

fn jdi_jar_path() -> PathBuf {
    if let Ok(java_home) = env::var("JAVA_HOME") {
        let path = PathBuf::from(format!("{}/lib/sa-jdi.jar", java_home));
        if !path.exists() {
            error_exit(&format!(
                "required sa-jdi.jar could not found in {:?}",
                path
            ));
        }
        return path;
    } else {
        error_exit("JAVA_HOME is not set");
    }
}

fn get_jvm_threads(pid: u32) -> Result<HashMap<u32, ThreadInfo>, jni::errors::Error> {
    let jvm_args = InitArgsBuilder::new()
        .option(&format!(
            "-Djava.class.path=./jthreadinfo/build/libs/jthreadinfo.jar:{}",
            jdi_jar_path().to_str().unwrap(),
        ))
        .build()
        .expect("build jvm args");
    let jvm = JavaVM::new(jvm_args)?;
    let env = jvm.attach_current_thread()?;

    let threads = env
        .call_static_method(
            "jthreadinfo/JThreadInfo",
            "listThreads",
            "(I)[Ljthreadinfo/ThreadInfo;",
            &[JValue::from(pid as i32)],
        )?
        .l()
        .expect("listThreads must return jobject")
        .into_inner() as jobjectArray;
    let len = env.get_array_length(threads)?;
    let mut mapping = HashMap::with_capacity(len as usize);
    for i in 0..len {
        let info = env.get_object_array_element(threads, i)?;
        let tid = env
            .get_field(info, "tid", "J")?
            .j()
            .expect("tid must be long");
        let nid = env
            .get_field(info, "nid", "J")?
            .j()
            .expect("nid must be long");
        let name = env
            .get_field(info, "name", "Ljava/lang/String;")?
            .l()
            .expect("name must be jobject");
        let name: String = env.get_string(name.into())?.into();
        let is_java = env
            .call_method(info, "isJavaThread", "()Z", &[])?
            .z()
            .expect("isJavaThread must return boolean");

        mapping.insert(nid as u32, ThreadInfo { tid, name, is_java });
    }

    Ok(mapping)
}

fn main() {
    let matches = App::new("A command line interface to Linux taskstats for JVM")
        .arg(Arg::with_name("verbose").short("v").long("verbose"))
        .arg(Arg::with_name("show-delays").short("d").long("delay"))
        .arg(Arg::with_name("JVM_PID").index(1))
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

    let tids: Vec<_> = mapping
        .iter()
        .filter_map(|(k, v)| if v.is_java { Some(*k) } else { None })
        .collect();
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
            format!("Thread {} (nid: {}) - {}", info.tid, tid, info.name)
        } else {
            format!("Thread UNKNOWN (nid: {})", tid)
        }
    }
}
