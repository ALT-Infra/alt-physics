use alt_graph_physics::{layout, Edge, EdgeKind, LayoutConfig, LayoutInput, Node, Pin, Port, Size};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn graph(count: usize) -> LayoutInput {
    let nodes = (0..count)
        .map(|id| Node {
            id: id as u64,
            size: Size::new(120.0, 64.0),
            pin: Pin::Free,
        })
        .collect();
    let edges = (1..count)
        .map(|target| Edge {
            id: target as u64,
            source: ((target - 1) / 2) as u64,
            target: target as u64,
            kind: EdgeKind::Directed {
                target_delta: 140.0,
            },
            ideal_length: 180.0,
            weight: 1.0,
            source_port: Port::Free,
            target_port: Port::Free,
        })
        .collect();
    LayoutInput {
        nodes,
        edges,
        constraints: vec![],
        config: LayoutConfig::default(),
    }
}

fn layout_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout");
    for count in [8, 24, 64] {
        let input = graph(count);
        group.bench_with_input(BenchmarkId::from_parameter(count), &input, |b, input| {
            b.iter(|| layout(input).unwrap());
        });
    }
    group.finish();
}

criterion_group!(benches, layout_benchmark);
criterion_main!(benches);
