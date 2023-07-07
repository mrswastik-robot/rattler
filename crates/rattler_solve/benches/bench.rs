use criterion::{black_box, criterion_group, criterion_main, Criterion, SamplingMode};
use rattler_conda_types::{Channel, ChannelConfig, MatchSpec};
use rattler_repodata_gateway::sparse::SparseRepoData;
use rattler_solve::{SolverImpl, SolverTask};
use std::str::FromStr;

fn conda_json_path() -> String {
    format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        "../../test-data/channels/conda-forge/linux-64/repodata.json"
    )
}

fn conda_json_path_noarch() -> String {
    format!(
        "{}/{}",
        env!("CARGO_MANIFEST_DIR"),
        "../../test-data/channels/conda-forge/noarch/repodata.json"
    )
}

fn read_sparse_repodata(path: &str) -> SparseRepoData {
    SparseRepoData::new(
        Channel::from_str("dummy", &ChannelConfig::default()).unwrap(),
        "dummy".to_string(),
        path,
    )
    .unwrap()
}

fn bench_solve_environment(c: &mut Criterion, specs: Vec<&str>) {
    let name = specs.join(", ");
    let mut group = c.benchmark_group(format!("solve {name}"));

    group.sampling_mode(SamplingMode::Flat);
    group.sample_size(10);

    let specs = specs
        .iter()
        .map(|s| MatchSpec::from_str(s).unwrap())
        .collect::<Vec<MatchSpec>>();

    let json_file = conda_json_path();
    let json_file_noarch = conda_json_path_noarch();

    let sparse_repo_datas = vec![
        read_sparse_repodata(&json_file),
        read_sparse_repodata(&json_file_noarch),
    ];

    let names = specs.iter().map(|s| s.name.clone().unwrap());
    let available_packages =
        SparseRepoData::load_records_recursive(&sparse_repo_datas, names).unwrap();

    #[cfg(feature = "libsolv-sys")]
    group.bench_function("libsolv-sys", |b| {
        b.iter(|| {
            rattler_solve::libsolv_sys::Solver
                .solve(black_box(SolverTask {
                    available_packages: &available_packages,
                    locked_packages: vec![],
                    pinned_packages: vec![],
                    virtual_packages: vec![],
                    specs: specs.clone(),
                }))
                .unwrap()
        })
    });

    #[cfg(feature = "libsolv_rs")]
    group.bench_function("libsolv_rs", |b| {
        b.iter(|| {
            rattler_solve::libsolv_rs::Solver
                .solve(black_box(SolverTask {
                    available_packages: &available_packages,
                    locked_packages: vec![],
                    pinned_packages: vec![],
                    virtual_packages: vec![],
                    specs: specs.clone(),
                }))
                .unwrap()
        })
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_solve_environment(c, vec!["python=3.9"]);
    bench_solve_environment(c, vec!["xtensor", "xsimd"]);
    bench_solve_environment(c, vec!["tensorflow"]);
    bench_solve_environment(c, vec!["quetz"]);
    bench_solve_environment(c, vec!["tensorboard=2.1.1", "grpc-cpp=1.39.1"]);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);