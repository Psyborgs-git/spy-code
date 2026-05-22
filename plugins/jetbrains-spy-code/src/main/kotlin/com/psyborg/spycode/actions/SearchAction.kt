package com.psyborg.spycode.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.psyborg.spycode.services.SpyCodeService

class SearchAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val service = SpyCodeService.getInstance(project)
        
        if (!service.isAvailable()) {
            Messages.showErrorDialog(project, "Spy-Code is not installed or not available", "Spy-Code Error")
            return
        }
        
        val query = Messages.showInputDialog(
            project,
            "Enter search query:",
            "Search Codebase",
            null
        ) ?: return
        
        val results = service.search(query)
        
        if (results.isEmpty()) {
            Messages.showInfoMessage(project, "No results found", "Search Results")
            return
        }
        
        // Show results in a dialog (simplified)
        val message = results.joinToString("\n") { "${it.node.name} - ${it.node.filePath}" }
        Messages.showInfoMessage(project, message, "Search Results")
    }
    
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null
    }
}
