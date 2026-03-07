use regex::Regex;

use crate::types::Language;

use super::SideEffect;

pub struct EffectTracker {
    patterns: Vec<EffectPattern>,
}

struct EffectPattern {
    pattern: Regex,
    effect_type: EffectType,
    languages: Vec<Language>,
}

enum EffectType {
    FileWrite,
    FileRead,
    FileDelete,
    Network,
    ProcessSpawn,
    EnvAccess,
}

impl EffectTracker {
    pub fn new() -> Self {
        let patterns = vec![
            // File write patterns
            EffectPattern {
                pattern: Regex::new(r#"(?:write|write_all|write_to_string|save|open\(.+["']w)"#)
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::FileWrite,
                languages: vec![],
            },
            EffectPattern {
                pattern: Regex::new(r"fs::write|std::fs::write|File::create")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::FileWrite,
                languages: vec![Language::Rust],
            },
            // File read patterns
            EffectPattern {
                pattern: Regex::new(r"fs::read|std::fs::read|File::open|read_to_string")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::FileRead,
                languages: vec![Language::Rust],
            },
            EffectPattern {
                pattern: Regex::new(r#"open\(.+["']r"#)
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::FileRead,
                languages: vec![Language::Python],
            },
            // File delete patterns
            EffectPattern {
                pattern: Regex::new(r"fs::remove|remove_file|remove_dir|os\.remove|unlink")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::FileDelete,
                languages: vec![],
            },
            // Network patterns
            EffectPattern {
                pattern: Regex::new(
                    r"TcpStream|UdpSocket|reqwest|hyper|socket|fetch\(|XMLHttpRequest|http\.get",
                )
                .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::Network,
                languages: vec![],
            },
            // Process spawn patterns
            EffectPattern {
                pattern: Regex::new(
                    r"Command::new|subprocess|os\.system|child_process|exec\(|spawn\(",
                )
                .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::ProcessSpawn,
                languages: vec![],
            },
            // Env access patterns
            EffectPattern {
                pattern: Regex::new(r"env::var|os\.environ|process\.env|getenv")
                    .unwrap_or_else(|_| Regex::new(r"$^").expect("fallback")),
                effect_type: EffectType::EnvAccess,
                languages: vec![],
            },
        ];

        Self { patterns }
    }

    pub fn analyze(&self, code: &str, language: &Language) -> Vec<SideEffect> {
        let mut effects = Vec::new();

        for pattern in &self.patterns {
            if !pattern.languages.is_empty() && !pattern.languages.contains(language) {
                continue;
            }

            for line in code.lines() {
                if pattern.pattern.is_match(line) {
                    let effect = match pattern.effect_type {
                        EffectType::FileWrite => SideEffect::FileWrite {
                            path: extract_path_hint(line),
                            bytes: 0,
                        },
                        EffectType::FileRead => SideEffect::FileRead {
                            path: extract_path_hint(line),
                        },
                        EffectType::FileDelete => SideEffect::FileDelete {
                            path: extract_path_hint(line),
                        },
                        EffectType::Network => SideEffect::NetworkConnect {
                            host: "unknown".to_string(),
                            port: 0,
                        },
                        EffectType::ProcessSpawn => SideEffect::ProcessSpawn {
                            command: line.trim().to_string(),
                        },
                        EffectType::EnvAccess => SideEffect::EnvAccess {
                            variable: extract_env_hint(line),
                        },
                    };
                    effects.push(effect);
                }
            }
        }

        effects
    }

    pub fn has_dangerous_effects(&self, code: &str, language: &Language) -> bool {
        self.analyze(code, language)
            .iter()
            .any(|e| e.is_dangerous())
    }
}

impl Default for EffectTracker {
    fn default() -> Self {
        Self::new()
    }
}

fn extract_path_hint(line: &str) -> String {
    let re = Regex::new(r#"["']([^"']+)["']"#).unwrap_or_else(|_| Regex::new(r"$^").expect("fallback"));
    re.captures(line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn extract_env_hint(line: &str) -> String {
    let re = Regex::new(r#"["']([A-Z_]+)["']"#).unwrap_or_else(|_| Regex::new(r"$^").expect("fallback"));
    re.captures(line)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
