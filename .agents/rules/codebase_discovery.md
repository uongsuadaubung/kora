# Codebase Discovery Rules

## Mandatory MCP Usage (codebase-memory-mcp)

- **Always use MCP**: ALWAYS use `codebase-memory-mcp` (codebasemap) for searching symbols, functions, classes, files, or tracing paths in code.
- **Tool priority**:
  1. `search_graph`: Find functions, structs, enums, routes by name pattern.
  2. `search_code`: Search text, code snippets, or file paths across the graph.
  3. `trace_path`: Trace callers/callees of functions.
  4. `get_code_snippet`: Read source code of specific symbols.
  5. `check_index_coverage`: Validate index coverage before conclusions.
- **No shell search commands**: NEVER run terminal commands (`Get-ChildItem`, `dir`, `find`, `grep`) to search or list source files unless explicitly requested by user or unsupported by MCP (binary/media assets).

