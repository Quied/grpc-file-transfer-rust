use assert_cmd::Command;
use bytesize::ByteSize;
use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use e2e_test_context::{AppType, E2ETestContext};
use predicates::prelude::predicate;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::path::{Path, PathBuf};

mod e2e_test_context;

fn list_files(cmd: &mut Command) {
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("File name  Size"))
        .stdout(predicate::str::contains("abc        5B"))
        .stdout(predicate::str::contains("xyz        4B"));
}

fn download_file(cmd: &mut Command, files: (&PathBuf, &PathBuf)) {
    cmd.assert().success();

    compare_files_on_disk(files.0, files.1);
}

fn upload_file(cmd: &mut Command, files: (&PathBuf, &PathBuf)) {
    cmd.assert().success();

    compare_files_on_disk(files.0, files.1);
}

fn compare_files_on_disk(left: &Path, right: &Path) {
    let status = std::process::Command::new("diff")
        .arg(left.as_os_str())
        .arg(right.as_os_str())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .expect("failed to execute diff process");

    if !status.success() {
        panic!("diff for file failed");
    }
}

fn criterion_benchmark_list_files(c: &mut Criterion) {
    let mut ctx = E2ETestContext::setup();
    ctx.start_server("::1".parse().unwrap());
    ctx.create_test_file(AppType::Server, "abc", "hello");
    ctx.create_test_file(AppType::Server, "xyz", "grpc");

    let mut cmd = Command::cargo_bin("client").unwrap();
    cmd.args(["--port", &ctx.port.to_string()])
        .args(["--address", "::1"])
        .arg("list");

    let mut group = c.benchmark_group("throughput-list_files");
    group.throughput(Throughput::Elements(1));
    group.bench_function("list_files", |b| b.iter(|| list_files(black_box(&mut cmd))));
    group.finish();
}

fn criterion_benchmark_download_file_param(c: &mut Criterion) {
    let mut ctx = E2ETestContext::setup();
    ctx.start_server("::1".parse().unwrap());

    let mut group = c.benchmark_group("throughput-download_file");

    for size in [ByteSize::mib(1), ByteSize::gib(1)] {
        let rand_content: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size.as_u64() as usize)
            .map(char::from)
            .collect();

        let file_name = format!("file_{size}");
        ctx.create_test_file(AppType::Server, &file_name, &rand_content);

        let file_server_path = ctx.server.files.last().unwrap().abs_path.clone();

        let mut file_client_path = PathBuf::new();
        file_client_path.push(ctx.client.dir.path());
        file_client_path.push(&file_name);

        let mut download_cmd = Command::cargo_bin("client").unwrap();
        download_cmd
            .args(["--port", &ctx.port.to_string()])
            .args(["--address", "::1"])
            .arg("download")
            .args(["--file", &file_name])
            .args(["--directory", ctx.client.dir.path().to_str().unwrap()]);

        group.throughput(Throughput::Bytes(size.as_u64()));
        group.sample_size(10);

        group.bench_function(format!("download_file_{}", size.to_string_as(true)), |b| {
            b.iter(|| {
                download_file(
                    black_box(&mut download_cmd),
                    (&file_server_path, &file_client_path),
                )
            })
        });
    }

    group.finish();
}

fn criterion_benchmark_upload_file_param(c: &mut Criterion) {
    let mut ctx = E2ETestContext::setup();
    ctx.start_server("::1".parse().unwrap());

    let mut group = c.benchmark_group("throughput-upload_file");

    for size in [ByteSize::mib(1), ByteSize::gib(1)] {
        let rand_content: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size.as_u64() as usize)
            .map(char::from)
            .collect();

        let file_name = format!("file_{size}");
        ctx.create_test_file(AppType::Client, &file_name, &rand_content);

        let mut file_server_path = PathBuf::new();
        file_server_path.push(ctx.server.dir.path());
        file_server_path.push(&file_name);

        let file_client_path = ctx.client.files.last().unwrap().abs_path.clone();

        let mut upload_cmd = Command::cargo_bin("client").unwrap();
        upload_cmd
            .args(["--port", &ctx.port.to_string()])
            .args(["--address", "::1"])
            .arg("upload")
            .args(["--file", &file_name])
            .args(["--directory", ctx.client.dir.path().to_str().unwrap()]);

        group.throughput(Throughput::Bytes(size.as_u64()));
        group.sample_size(10);

        group.bench_function(format!("upload_file_{}", size.to_string_as(true)), |b| {
            b.iter(|| {
                upload_file(
                    black_box(&mut upload_cmd),
                    (&file_server_path, &file_client_path),
                )
            })
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    criterion_benchmark_list_files,
    criterion_benchmark_download_file_param,
    criterion_benchmark_upload_file_param
);
criterion_main!(benches);
