//! Collects the built-in Fluent locale bundles shipped with the Souprune editor.
//!
//! 汇总随 Souprune 编辑器一起分发的内置 Fluent 本地化资源。
//!
//! Acts as the narrow entry for editor translation data. It keeps the
//! concrete language bundles in separate files while exposing the small set of
//! constants used by the i18n registration code.
//!
//! 编辑器翻译资源的窄入口。具体语言包拆在独立文件里，而这里负责
//! 暴露国际化注册代码真正需要消费的那几个常量。

mod locale_sources_en;
mod locale_sources_zh_cn;

pub use self::locale_sources_en::EN_FTL;
pub use self::locale_sources_zh_cn::ZH_CN_FTL;
