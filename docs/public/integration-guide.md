---
status: stable
---

# Integration Guide

AgenticAegis integrates with other Agentra sisters through trait-based bridges.

## Bridge Architecture

Each sister exposes a bridge trait with NoOp defaults. AgenticAegis can operate standalone or connect to siblings like AgenticMemory, AgenticCognition, or AgenticReality.

## MCP Integration

Connect any MCP-compatible client to the AgenticAegis MCP server. The 12 tools cover validation, execution, session management, and security scanning.

## SDK Integration

Reference `agentic-sdk v0.2.0` for unified access to all sister capabilities through a single API surface.

## Standalone Guarantee

AgenticAegis works independently without any other sister installed. Bridge connections are optional enhancements, not requirements.
