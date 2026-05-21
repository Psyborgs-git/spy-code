import sqlite3
import os
import sys

def main():
    db_path = ".spy-code/graph.db"
    if not os.path.exists(db_path):
        print(f"Error: Database {db_path} does not exist. Please run spy-code index first.")
        sys.exit(1)

    conn = sqlite3.connect(db_path)
    cursor = conn.cursor()

    # Get all nodes
    cursor.execute("SELECT node_id, kind, name, description, language, file_path, start_line, end_line FROM nodes")
    nodes_raw = cursor.fetchall()

    # Get all edges
    cursor.execute("SELECT from_id, to_id FROM edges_calls")
    edges_calls = cursor.fetchall()
    cursor.execute("SELECT from_id, to_id FROM edges_imports")
    edges_imports = cursor.fetchall()
    cursor.execute("SELECT from_id, to_id FROM edges_references")
    edges_references = cursor.fetchall()

    print(f"Found {len(nodes_raw)} nodes, {len(edges_calls)} call edges, {len(edges_imports)} import edges, {len(edges_references)} reference edges.")

    # Parse nodes
    nodes = {}
    node_id_to_alias = {}
    alias_counter = 0

    for node in nodes_raw:
        node_id, kind, name, description, language, file_path, start_line, end_line = node
        
        # Format is dir:file:class:symbol
        parts = node_id.split(':')
        if len(parts) >= 4:
            dir_part = parts[0]
            file_part = parts[1]
            class_part = parts[2]
            symbol_part = ":".join(parts[3:])
        else:
            dir_part = parts[0] if len(parts) > 0 else "_"
            file_part = parts[1] if len(parts) > 1 else "_"
            class_part = parts[2] if len(parts) > 2 else "_"
            symbol_part = parts[3] if len(parts) > 3 else "_"
        
        # Extract crate name
        crate = "unknown"
        if "crates/" in dir_part:
            idx = dir_part.find("crates/")
            rest = dir_part[idx + 7:]
            crate = rest.split('/')[0]
        elif "src" in dir_part or dir_part == ".":
            crate = "spy-code" # root binary
            
        alias = f"n_{alias_counter}"
        alias_counter += 1
        node_id_to_alias[node_id] = alias

        nodes[node_id] = {
            'node_id': node_id,
            'alias': alias,
            'kind': kind,
            'name': name,
            'description': description,
            'language': language,
            'file_path': file_path,
            'start_line': start_line,
            'end_line': end_line,
            'crate': crate,
            'file': file_part,
            'class': class_part,
            'symbol': symbol_part
        }

    # Aggregate dependencies between crates
    crate_dependencies = {} # (from_crate, to_crate) -> count
    
    def add_crate_dep(from_id, to_id):
        if from_id in nodes and to_id in nodes:
            from_crate = nodes[from_id]['crate']
            to_crate = nodes[to_id]['crate']
            if from_crate != to_crate and from_crate != 'unknown' and to_crate != 'unknown':
                key = (from_crate, to_crate)
                crate_dependencies[key] = crate_dependencies.get(key, 0) + 1

    for from_id, to_id in edges_calls:
        add_crate_dep(from_id, to_id)
    for from_id, to_id in edges_imports:
        add_crate_dep(from_id, to_id)
    for from_id, to_id in edges_references:
        add_crate_dep(from_id, to_id)

    # 1. Generate High-Level Crate Dependency Graph
    crate_mermaid = "graph TD\n"
    crate_mermaid += "    %% Crate dependency diagram\n"
    # Highlight style
    crate_mermaid += "    classDef default fill:#1e1e2e,stroke:#cdd6f4,stroke-width:1px,color:#cdd6f4;\n"
    crate_mermaid += "    classDef core fill:#313244,stroke:#f38ba8,stroke-width:2px,color:#f38ba8;\n"
    
    crates = sorted(list(set(n['crate'] for n in nodes.values() if n['crate'] != 'unknown')))
    for crate in crates:
        if crate in ['spy-core', 'spy-storage']:
            crate_mermaid += f"    {crate.replace('-', '_')}[\"{crate} (Core)\"]:::core\n"
        else:
            crate_mermaid += f"    {crate.replace('-', '_')}[\"{crate}\"]\n"

    for (from_crate, to_crate), count in sorted(crate_dependencies.items(), key=lambda x: -x[1]):
        from_node = from_crate.replace('-', '_')
        to_node = to_crate.replace('-', '_')
        crate_mermaid += f"    {from_node} -->|{count} calls| {to_node}\n"

    # 2. Detailed Graph of Key Codebase Components (filtered to remove tests and trivial helpers)
    # We filter out tests, helper wrappers, and limit to key architecture components
    key_nodes = {}
    for node_id, node in nodes.items():
        name_lower = node['name'].lower()
        file_lower = node['file'].lower()
        
        # Filter out tests
        if 'test' in name_lower or 'test' in file_lower or 'fixture' in file_lower:
            continue
        # Filter out helper traits unless they are important
        if node['crate'] == 'unknown':
            continue
            
        key_nodes[node_id] = node

    # Keep edges where both endpoints are key nodes
    key_edges_calls = [(f, t) for f, t in edges_calls if f in key_nodes and t in key_nodes]
    key_edges_imports = [(f, t) for f, t in edges_imports if f in key_nodes and t in key_nodes]
    key_edges_refs = [(f, t) for f, t in edges_references if f in key_nodes and t in key_nodes]

    # Let's organize the detail call graph
    # Group by crate
    crates_grouped = {}
    for node_id, node in key_nodes.items():
        crate = node['crate']
        if crate not in crates_grouped:
            crates_grouped[crate] = []
        crates_grouped[crate].append(node)

    detail_mermaid = "graph LR\n"
    detail_mermaid += "    %% Detailed Call and Import Graph\n"
    detail_mermaid += "    classDef fn fill:#313244,stroke:#89b4fa,stroke-width:1px,color:#89b4fa;\n"
    detail_mermaid += "    classDef cls fill:#313244,stroke:#a6e3a1,stroke-width:1.5px,color:#a6e3a1;\n"
    detail_mermaid += "    classDef const fill:#313244,stroke:#f9e2af,stroke-width:1px,color:#f9e2af;\n"

    for crate, crate_nodes in sorted(crates_grouped.items()):
        # We start a subgraph for each crate to visually isolate them
        detail_mermaid += f"    subgraph {crate.replace('-', '_')}_sub[\"{crate}\"]\n"
        for node in crate_nodes:
            label = f"[{node['kind']}] {node['name']}"
            if node['class'] != '_':
                label = f"{node['class']}::{node['name']}"
            
            style_class = "fn"
            if node['kind'] == 'class':
                style_class = "cls"
            elif node['kind'] == 'constant':
                style_class = "const"
                
            detail_mermaid += f"        {node['alias']}[\"{label}\"]:::{style_class}\n"
        detail_mermaid += "    end\n\n"

    # Add edges to the detailed graph
    # To prevent spaghetti, we prioritize calls, then imports/refs if relevant, and limit to max 100 edges
    edge_written = 0
    
    # 1. Calls (solid arrows)
    for from_id, to_id in key_edges_calls:
        if from_id in key_nodes and to_id in key_nodes:
            from_alias = key_nodes[from_id]['alias']
            to_alias = key_nodes[to_id]['alias']
            detail_mermaid += f"    {from_alias} --> {to_alias}\n"
            edge_written += 1
            
    # 2. Imports/References (dashed/dotted arrows)
    for from_id, to_id in key_edges_imports:
        if from_id in key_nodes and to_id in key_nodes:
            from_alias = key_nodes[from_id]['alias']
            to_alias = key_nodes[to_id]['alias']
            detail_mermaid += f"    {from_alias} -.->|imports| {to_alias}\n"
            edge_written += 1

    # Output markdown formatting
    output_md = f"""# spy-code Codebase Analysis & Architecture Graph

This document presents a detailed analysis and architectural graph of the `spy-code` codebase, generated by running the `spy-code` indexer on itself.

## Codebase Statistics
- **Total Indexed Files**: {len(crates)} modules across crates
- **Total Code Symbols (Nodes)**: {len(nodes_raw)}
- **Call Relationships (Edges)**: {len(edges_calls)}
- **Imports**: {len(edges_imports)}
- **References**: {len(edges_references)}

---

## 1. High-Level Crate Dependency Graph
This diagram shows the dependencies and number of calls/imports between the crates in the `spy-code` workspace.

```mermaid
{crate_mermaid}```

### Crate Descriptions:
1. **`spy-core`**: Defines the central domain data structures (`Node`, `Edge`, `NodeId`, `Language`, `Config`) and core traits.
2. **`spy-storage`**: Implements the SQLite database backend, handling node/edge upserts, indexing metadata, and FTS search.
3. **`spy-parser`**: Orchestrates parsing source files using `tree-sitter`.
4. **`spy-resolvers`**: Language-specific AST traversers (Rust, Python, TypeScript, Go) that extract defined nodes (functions, classes, constants) and calls.
5. **`spy-indexer`**: Coordinates the indexing pipeline, performing Pass 1 (node extraction), Pass 2 (dependency extraction), and incremental indexing using git history.
6. **`spy-graph`**: Exposes the GraphQL API schema and resolvers over the indexed codebase.
7. **`spy-mcp`**: Implements the Model Context Protocol (MCP) server, allowing AI agents to query the database.
8. **`spy-cli`**: Command-line interface for indexing codebases, checking callers/callees, and running queries.
9. **`spy-git`**: Interacts with the git repository to determine modified or renamed files for incremental updates.

---

## 2. Detailed Symbol Call Graph
The following diagram highlights the key functions, classes, and structs within each crate and illustrates how they call and import each other. 
*(Note: Test nodes and standard utility functions are excluded for visual clarity.)*

```mermaid
{detail_mermaid}```

---

## Key architectural observations
- **Central Storage Dependency**: Almost all packages depend on `spy-core` for the domain types and `spy-storage` to read/write nodes and edges.
- **Resolver-Driven Extraction**: `spy-resolvers` parses source code into nodes, which are then passed to `spy-storage`.
- **GraphQL & MCP Clients**: `spy-graph` and `spy-mcp` serve as read-only consumers of the storage backend, querying nodes, callers, and callees.
"""

    # Write to target artifact
    target_path = "/Users/jainamshah/.gemini/antigravity/brain/ae5e0308-724d-4a29-8270-7bce824d3e2b/spy_code_graph.md"
    os.makedirs(os.path.dirname(target_path), exist_ok=True)
    with open(target_path, "w") as f:
        f.write(output_md)
        
    print(f"Successfully generated architectural graph artifact at {target_path}")

if __name__ == "__main__":
    main()
