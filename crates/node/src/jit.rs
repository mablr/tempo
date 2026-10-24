//! Native EVM2 JIT backend for Tempo nodes.

use reth_node_builder::{BuilderContext, FullNodeTypes};
use reth_tracing::tracing::{debug, info, warn};
use std::{sync::Arc, time::Duration};
use tempo_chainspec::hardfork::TempoHardforks;
use tempo_evm::jit::{JitBackend, JitMetrics, RuntimeConfig, RuntimeTuning, runtime_config};

/// Starts a JIT backend for the chain's latest hardfork from the node's JIT arguments.
///
/// Frames under earlier hardforks stay on the interpreter.
pub(crate) fn spawn_backend<Node>(ctx: &BuilderContext<Node>) -> eyre::Result<JitBackend>
where
    Node: FullNodeTypes<Types = crate::TempoNode>,
{
    let args = &ctx.config().jit;
    let spec = ctx.chain_spec().tempo_hardfork_at(u64::MAX);
    let defaults = RuntimeTuning::default();
    let metrics = JitMetrics::default();
    let compilation_metrics = metrics.clone();
    let backend = JitBackend::new(RuntimeConfig {
        enabled: true,
        blocking: args.blocking,
        debug_assertions: args.debug,
        dump_dir: args
            .debug
            .then(|| ctx.config().datadir().data_dir().join("jit")),
        tuning: RuntimeTuning {
            channel_capacity: args.channel_capacity,
            jit_hot_threshold: args.hot_threshold,
            jit_max_bytecode_len: args.max_bytecode_len,
            jit_max_pending_jobs: args.max_pending_jobs,
            jit_worker_count: args.worker_count.unwrap_or(defaults.jit_worker_count),
            resident_code_cache_bytes: args.code_cache_bytes,
            idle_evict_duration: Some(args.idle_evict_duration),
            ..defaults
        },
        on_compilation: Some(Arc::new(move |event| {
            compilation_metrics.record_compilation(&event);
            debug!(target: "tempo::jit", code_hash = %event.code_hash, success = event.success,
                duration = ?event.duration, "JIT compilation finished");
        })),
        ..runtime_config(spec)
    })?;
    warn!(target: "tempo::jit", ?spec, hot_threshold = args.hot_threshold,
        blocking = args.blocking, "Started experimental EVM2 JIT backend");

    let stats_backend = backend.clone();
    ctx.task_executor()
        .spawn_with_graceful_shutdown_signal(|shutdown| async move {
            let mut shutdown = std::pin::pin!(shutdown);
            loop {
                tokio::select! {
                    _ = tokio::time::sleep(Duration::from_secs(5)) => {
                        let stats = stats_backend.stats();
                        metrics.record(&stats);
                        info!(target: "tempo::jit", hits = stats.lookup_hits,
                            misses = stats.lookup_misses,
                            compiled = stats.compilations_succeeded,
                            failed = stats.compilations_failed, "JIT runtime stats");
                    }
                    _ = &mut shutdown => break,
                }
            }
        });
    Ok(backend)
}
