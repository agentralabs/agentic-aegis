---
status: stable
---

# Frequently Asked Questions

## What is AgenticAegis?

AgenticAegis is a streaming validation and security protection system for AI-generated code. It validates code in real time as tokens are generated.

## Does it require other Agentra sisters?

No. AgenticAegis operates fully standalone. Sister bridges are optional.

## What languages does it validate?

AgenticAegis validates any text-based code output. Language-specific analyzers provide deeper checks for Python, Rust, JavaScript, and TypeScript.

## How fast is streaming validation?

Sub-millisecond per token on modern hardware. See the benchmarks page for detailed numbers.

## Can I use it with my own LLM?

Yes. AgenticAegis works with any LLM or agent that produces code output, whether via MCP, CLI piping, or the FFI bindings.
