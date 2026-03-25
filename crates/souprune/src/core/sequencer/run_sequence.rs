//! Collects the runtime pieces for `RunSequence` chapters.
//!
//! 汇总 `RunSequence` 章节所需的运行时部件。
//!
//! A `RunSequence` chapter is different from ordinary chapter execution because
//! it has to resolve another sequence asset, optionally inject parameters, and
//! splice the loaded chapters back into the current queue. This module gathers
//! the loading and parameter-handling logic for that behavior.
//!
//! `RunSequence` 章节和普通章节不同：它需要解析另一份序列资源，可选地注入参数，
//! 再把加载出的章节重新拼回当前执行队列。这个模块把这种行为涉及的加载与参数
//! 处理逻辑集中在一起。

mod loading;
mod params;

pub use loading::{complete_run_sequence_system, process_run_sequence_system};
