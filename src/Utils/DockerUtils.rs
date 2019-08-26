use std::io::Read;

use dockworker::{container::ContainerFilters, CreateExecOptions, Docker, StartExecOptions};

pub fn GetContainers(docker: &Docker) -> Vec<String> {
    let filter = ContainerFilters::new();
    let containers = docker
        .list_containers(Some(true), None, None, filter)
        .unwrap();
    let mut ret = Vec::new();
    for i in &containers {
        ret.push(i.Id.clone());
    }
    return ret;
}

pub fn RunCmd(id: &str, cmd: String) -> (u32, String) {
    let docker = Docker::connect_with_defaults().unwrap();
    let mut buf: Vec<u8> = Vec::new();
    let idx = docker
        .exec_container(
            id,
            &CreateExecOptions::new()
                .tty(true)
                .cmd("sh".to_string())
                .cmd("-c".to_string())
                .cmd(cmd),
        )
        .unwrap()
        .id;
    docker
        .start_exec(&idx, &StartExecOptions::new().tty(true))
        .unwrap()
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();
    let status = docker.exec_inspect(&idx).unwrap().ExitCode.unwrap();
    let info = String::from_utf8(buf).unwrap();
    (status, info)
}