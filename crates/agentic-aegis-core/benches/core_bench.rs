use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

use agentic_aegis_core::cache::LruCache;
use agentic_aegis_core::types::{
    Language, SessionConfig, SessionId, ValidationContext, ValidationSession,
};

fn bench_session_creation(c: &mut Criterion) {
    c.bench_function("session_creation", |b| {
        b.iter(|| {
            let config = SessionConfig {
                language: Language::Rust,
                file_path: Some("bench.rs".into()),
                max_errors: 50,
                ..Default::default()
            };
            let session = ValidationSession::new(config);
            black_box(session.id.to_string());
        })
    });
}

fn bench_session_activate(c: &mut Criterion) {
    c.bench_function("session_activate", |b| {
        b.iter(|| {
            let config = SessionConfig {
                language: Language::Rust,
                ..Default::default()
            };
            let mut session = ValidationSession::new(config);
            let _ = session.activate();
            black_box(&session.state);
        })
    });
}

fn bench_session_snapshot(c: &mut Criterion) {
    c.bench_function("session_snapshot", |b| {
        b.iter(|| {
            let config = SessionConfig {
                language: Language::Rust,
                ..Default::default()
            };
            let mut session = ValidationSession::new(config);
            session.context.accumulated_code = "fn main() { println!(\"hello\"); }".into();
            session.context.chunk_index = 5;
            session.take_snapshot();
            black_box(session.snapshots.len());
        })
    });
}

fn bench_validation_context_append(c: &mut Criterion) {
    c.bench_function("validation_context_append_chunk", |b| {
        b.iter(|| {
            let mut ctx =
                ValidationContext::new(SessionId::new(), Language::Rust, "bench.rs".into());
            for _ in 0..50 {
                ctx.append_chunk(black_box("let x = 42;\n"));
            }
            black_box(ctx.accumulated_code.len());
        })
    });
}

fn bench_cache_insert(c: &mut Criterion) {
    c.bench_function("cache_insert_100", |b| {
        b.iter(|| {
            let cache: LruCache<String, String> = LruCache::new(256, Duration::from_secs(300));
            for i in 0..100 {
                cache.insert(format!("key_{}", i), format!("val_{}", i));
            }
            black_box(cache.len());
        })
    });
}

fn bench_cache_get_hit(c: &mut Criterion) {
    let cache: LruCache<String, String> = LruCache::new(256, Duration::from_secs(300));
    for i in 0..100 {
        cache.insert(format!("key_{}", i), format!("val_{}", i));
    }

    c.bench_function("cache_get_hit", |b| {
        b.iter(|| {
            let result = cache.get(black_box(&"key_50".to_string()));
            black_box(result);
        })
    });
}

fn bench_cache_get_miss(c: &mut Criterion) {
    let cache: LruCache<String, String> = LruCache::new(256, Duration::from_secs(300));
    for i in 0..100 {
        cache.insert(format!("key_{}", i), format!("val_{}", i));
    }

    c.bench_function("cache_get_miss", |b| {
        b.iter(|| {
            let result = cache.get(black_box(&"nonexistent".to_string()));
            black_box(result);
        })
    });
}

fn bench_language_parse(c: &mut Criterion) {
    c.bench_function("language_from_str_loose", |b| {
        let inputs = [
            "rust",
            "python",
            "javascript",
            "typescript",
            "go",
            "java",
            "unknown_lang",
        ];
        b.iter(|| {
            for input in &inputs {
                black_box(Language::from_str_loose(input));
            }
        })
    });
}

fn bench_session_error_limit_check(c: &mut Criterion) {
    c.bench_function("session_error_limit_check", |b| {
        b.iter(|| {
            let config = SessionConfig {
                language: Language::Rust,
                max_errors: 50,
                ..Default::default()
            };
            let mut session = ValidationSession::new(config);
            session.total_errors = black_box(49);
            let result = session.is_over_error_limit();
            black_box(result);
        })
    });
}

criterion_group!(
    benches,
    bench_session_creation,
    bench_session_activate,
    bench_session_snapshot,
    bench_validation_context_append,
    bench_cache_insert,
    bench_cache_get_hit,
    bench_cache_get_miss,
    bench_language_parse,
    bench_session_error_limit_check,
);
criterion_main!(benches);
