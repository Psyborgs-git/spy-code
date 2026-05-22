package com.psyborg.spycode.services

import com.google.gson.Gson
import com.google.gson.JsonObject
import com.intellij.openapi.components.Service
import com.intellij.openapi.components.service
import com.intellij.openapi.project.Project
import java.io.BufferedReader
import java.io.InputStreamReader

@Service
class SpyCodeService(private val project: Project) {
    
    private val gson = Gson()
    private var mcpProcess: Process? = null
    
    fun isAvailable(): Boolean {
        return try {
            val process = ProcessBuilder("spy-code", "--version").start()
            process.waitFor() == 0
        } catch (e: Exception) {
            false
        }
    }
    
    fun search(query: String, kind: String? = null): List<SearchResult> {
        val response = executeMcpTool("search", mapOf(
            "query" to query,
            "kind" to (kind ?: ""),
            "limit" to 20
        ))
        
        return parseSearchResults(response)
    }
    
    fun getNode(nodeId: String): Node? {
        val response = executeMcpTool("get_node", mapOf("node_id" to nodeId))
        return parseNode(response)
    }
    
    fun getCallers(nodeId: String, depth: Int = 1): List<Edge> {
        val response = executeMcpTool("find_callers", mapOf(
            "node_id" to nodeId,
            "depth" to depth
        ))
        return parseEdges(response)
    }
    
    fun getCallees(nodeId: String, depth: Int = 1): List<Edge> {
        val response = executeMcpTool("find_callees", mapOf(
            "node_id" to nodeId,
            "depth" to depth
        ))
        return parseEdges(response)
    }
    
    fun getStats(): IndexStats {
        val response = executeMcpTool("stats", emptyMap())
        return parseStats(response)
    }
    
    fun index() {
        executeMcpTool("index", emptyMap())
    }
    
    private fun executeMcpTool(toolName: String, params: Map<String, Any>): String {
        // For now, use CLI commands directly
        // In production, this would use the MCP server
        return when (toolName) {
            "search" -> executeCliCommand("search", params["query"] as String)
            "get_node" -> executeCliCommand("get", params["node_id"] as String)
            "find_callers" -> executeCliCommand("callers", params["node_id"] as String)
            "find_callees" -> executeCliCommand("callees", params["node_id"] as String)
            "stats" -> executeCliCommand("stats")
            "index" -> executeCliCommand("index")
            else -> ""
        }
    }
    
    private fun executeCliCommand(vararg args: String): String {
        val command = mutableListOf("spy-code")
        command.addAll(args)
        
        val process = ProcessBuilder(command)
            .directory(project.baseDir)
            .start()
        
        val output = process.inputStream.bufferedReader().use { it.readText() }
        process.waitFor()
        
        return output
    }
    
    private fun parseSearchResults(json: String): List<SearchResult> {
        // Parse CLI output (simplified)
        return emptyList()
    }
    
    private fun parseNode(json: String): Node? {
        // Parse CLI output (simplified)
        return null
    }
    
    private fun parseEdges(json: String): List<Edge> {
        // Parse CLI output (simplified)
        return emptyList()
    }
    
    private fun parseStats(json: String): IndexStats {
        // Parse CLI output (simplified)
        return IndexStats(0, 0, 0)
    }
    
    data class SearchResult(val node: Node, val score: Double)
    data class Node(
        val id: String,
        val name: String,
        val kind: String,
        val filePath: String,
        val description: String?
    )
    data class Edge(val from: Node, val to: Node, val kind: String, val confidence: Double)
    data class IndexStats(val nodeCount: Int, val edgeCount: Int, val fileCount: Int)
    
    companion object {
        fun getInstance(project: Project): SpyCodeService {
            return project.service()
        }
    }
}
