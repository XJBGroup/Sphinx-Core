use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use dockworker::Docker;

use super::Compiler::{Compiler, CompileStatus};
use super::Env::*;
use super::Judge::Judge;
use super::Judge::JudgeResult;
use super::Judge::JudgeStatus;
use super::Language::language;
use super::SphinxCore::Judge::JudgeOption;

pub fn CopyFiles(
    docker: &Docker,
    id: &str,
    code: &String,
    index: &u32,
    lang: language,
) -> Result<(), String> {
    let dir = format!("{}/{}", WORK_DIR, index);
    let pdir = Path::new(&dir);
    if !pdir.exists() && fs::create_dir_all(pdir).is_err() {
        return Err(format!("make dir failed"));
    }
    let file = File::create(format!("{}/{}/Main.{}", WORK_DIR, index, lang.extension()));
    if file.is_err() {
        return Err("make file failed".to_string());
    }
    match file.unwrap().write_all(code.as_bytes()) {
        Ok(T) => Ok(()),
        Err(T) => Err("write file failed".to_string()),
    }
}

pub fn Run(
    docker: &Docker,
    ContainerId: &str,
    SubmissionId: &u32,
    DataUID: &str,
    lang: language,
    SpecialJudge: bool,
    opt: &JudgeOption,
    code: &String,
) -> JudgeResult {
    match CopyFiles(docker, ContainerId, code, SubmissionId, lang.clone()) {
        Ok(T) => {
            if lang.compile() {
                let res = Compiler(docker, ContainerId, SubmissionId, lang.clone());
                if res.status == CompileStatus::FAILED {
                    JudgeResult {
                        status: JudgeStatus::COMPILE_ERROR,
                        info: Some(res.info),
                        time_cost: 0,
                        memory_cost: 0,
                        last: 0,
                    }
                } else {
                    Judge(
                        docker,
                        ContainerId,
                        SubmissionId,
                        DataUID,
                        lang.clone(),
                        opt,
                        SpecialJudge,
                    )
                }
            } else {
                Judge(
                    docker,
                    ContainerId,
                    SubmissionId,
                    DataUID,
                    lang.clone(),
                    opt,
                    SpecialJudge,
                )
            }
        }
        Err(T) => JudgeResult {
            status: JudgeStatus::COMPILE_ERROR,
            info: Some(T),
            time_cost: 0,
            memory_cost: 0,
            last: 0,
        },
    }
}
