use super::DockerUtils;
use dockworker::{
    container::ContainerFilters, ContainerCreateOptions, CreateExecOptions, CreateExecResponse,
    Docker, StartExecOptions,
};
pub fn CopyFiles(docker: &Docker, id: &str, code: &String, index: &u32) {
    println!(
        "{}",
        DockerUtils::RunCmd(id, format!("echo \"{}\" > /code/{}.cpp", code, index), 1000)
            .expect("Copy Code File Failed")
    );
}

pub fn compiler(docker: &Docker, id: &str, index: &u32) {}
