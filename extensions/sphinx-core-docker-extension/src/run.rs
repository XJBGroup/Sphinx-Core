extern crate tar;

use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use dockworker::Docker;
use tar::Builder;

use sphinx_core::{Compiler, CompilerConfig, CompileStatus, Judge, JudgeReply, JudgeStatus, Language, MainServerClient, ProblemConfig};

use crate::utils::{create_judge_container, remove_judge_container};

use super::env::*;

pub fn copy_files(
    docker: &Docker,
    container_id: &str,
    uid: u64,
    code: String,
    judge_opt: &ProblemConfig,
    lang: Language,
    base_url: &str,
) -> Result<(), String> {
    // Write code into Temp Dir
    let dir_path = format!("{}/{}", WORK_DIR, uid);
    let pdir = Path::new(&dir_path);
    if !pdir.exists() && fs::create_dir_all(pdir).is_err() {
        return Err("make dir failed".to_string());
    }
    let code_path = format!("{}/{}/Main.{}", WORK_DIR, uid, lang.extension());
    let file = File::create(&code_path);
    if file.is_err() {
        return Err("make file failed".to_string());
    }
    match file.unwrap().write_all(code.as_bytes()) {
        Ok(_) => {}
        Err(err) => return Err(format!("write file failed,{}", err)),
    };
    // Copy Jury , code and Core into Docker
    let tar_path = format!("{}/{}/foo.tar", WORK_DIR, uid);
    let file = File::create(&tar_path).unwrap();
    let mut a = Builder::new(file);

    a.append_file(
        format!("Main.{}", lang.extension()),
        &mut File::open(&code_path).unwrap(),
    )
        .unwrap();
    if judge_opt.spj == NORMAL_JUDGE {
        a.append_file("judger", &mut File::open(&JURY).unwrap())
            .unwrap();
    } else {
        a.append_file(
            "judger",
            &mut File::open(&format!("{}/{}", base_url, judge_opt.spj_path)).unwrap(),
        )
            .unwrap();
    }
    if judge_opt.spj != INTERACTIVE_JUDGE {
        a.append_file("core", &mut File::open(CORE1).unwrap())
            .unwrap();
    } else {
        a.append_file("core", &mut File::open(CORE2).unwrap())
            .unwrap();
    }

    docker
        .put_file(container_id, &Path::new(&tar_path), Path::new("/tmp"), true)
        .unwrap();
    Ok(())
}

pub async fn run<T: MainServerClient>(
    docker: &Docker,
    submission_id: u64,
    lang: Language,
    judge_opt: ProblemConfig,
    code: String,
    base_url: &str,
    client: &mut T,
)
{
    let container_id = create_judge_container(&docker, base_url).unwrap();

    let cfg = CompilerConfig {};
    let mut compiler = crate::Compiler::new(&docker);
    compiler.config(&cfg);

    match copy_files(
        &docker,
        &container_id,
        submission_id,
        code,
        &judge_opt,
        lang.clone(),
        base_url,
    ) {
        Ok(_) => {
            if lang.compile() {
                let res = compiler.compile(&container_id, "/tmp".to_string(), lang.clone());
                if res.status == CompileStatus::FAILED {
                    client.update_real_time_info(&JudgeReply {
                        status: "COMPILE ERROR",
                        mem: 0,
                        time: 0,
                        submission_id,
                        last: 0,
                        score: 0,
                        info: &res.info,
                    }).await;
                    return;
                }
            }
            judge(
                &docker,
                &container_id,
                submission_id,
                &judge_opt,
                lang.clone(),
                base_url,
                client,
            ).await;
        }
        Err(err) => {
            client.update_real_time_info(&JudgeReply {
                status: "COMPILE ERROR",
                mem: 0,
                time: 0,
                submission_id,
                last: 0,
                score: 0,
                info: &err,
            }).await;
        }
    }

    remove_judge_container(&docker, &container_id).unwrap();
}

fn get_data(dir: &str, suf: &str) -> Vec<String> {
    println!("{}", dir);
    let path = Path::new(dir);
    let mut ret = Vec::new();
    for entry in path.read_dir().expect("read_dir call failed") {
        if let Ok(entry) = entry {
            let buf = entry.path();
            let prefix = buf.file_name().unwrap().to_str().unwrap();
            let suffix = buf.extension();
            if suffix.is_some() && suffix.unwrap().to_str().unwrap() == suf {
                ret.push(prefix.to_string().replace(&format!(".{}", suf), ""));
            }
        }
    }
    ret.sort();
    ret
}

pub async fn judge<T: MainServerClient>(
    docker: &Docker,
    container_id: &str,
    uid: u64,
    judge_opt: &ProblemConfig,
    lang: Language,
    base_url: &str,
    client: &mut T,
)
{
    let inner_judge = crate::Judge { docker: &docker, container_id: &container_id };

    let acm = judge_opt.judge_type == "acm";
    let is_interactive = judge_opt.spj == INTERACTIVE_JUDGE;
    let mut last: u32 = 0;

    for i in judge_opt.tasks.iter() {
        let input = get_data(&format!("{}/{}", base_url, i.input), "in");
        let output = if is_interactive {
            Vec::new()
        } else {
            get_data(&format!("{}/{}", base_url, i.output), "out")
        };
        if !is_interactive && input != output {
            client.update_real_time_info(&JudgeReply {
                status: "DATA INVALID",
                mem: 0,
                time: 0,
                submission_id: uid,
                last: 0,
                score: 0,
                info:
                "input output mismatch",
            }).await;
            return;
        }
        let mut max_t = 0;
        let mut max_m = 0;
        let mut max_status = JudgeStatus::Accepted;
        let mut max_score = 0;
        for j in 0..input.len() {
            let (status, _t, _m) = inner_judge.judge(
                i,
                &input[j],
                if is_interactive { "" } else { &output[j] },
                lang.clone(),
                is_interactive,
            );
            max_t = max_t.max(_t);
            max_m = max_m.max(_m);
            if status == JudgeStatus::Accepted {
                max_score += i.score;
                client.update_real_time_info(&JudgeReply {
                    status: "RUNNING",
                    mem: max_m,
                    time: max_t,
                    submission_id: uid,
                    last: last,
                    score: if acm { 0 } else { max_score },
                    info: "",
                }).await;
                last += 1;
            } else {
                max_status = status;
                client.update_real_time_info(&JudgeReply {
                    status: status.to_string(),
                    mem: max_m,
                    time: max_t,
                    submission_id: uid,
                    last: last,
                    score: if acm { 0 } else { max_score },
                    info: "",
                }).await;
                if acm {
                    return;
                }
            }
        }
        client.update_real_time_info(&JudgeReply {
            status: max_status.to_string(),
            mem: max_m,
            time: max_t,
            submission_id: uid,
            last: last,
            score: 0,
            info: "",
        }).await;
        last += 1;
    }
}
