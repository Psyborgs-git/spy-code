package com.psyborg.spycode.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.psyborg.spycode.services.SpyCodeService

class FindCallersAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val service = SpyCodeService.getInstance(project)
        
        if (!service.isAvailable()) {
            Messages.showErrorDialog(project, "Spy-Code is not installed or not available", "Spy-Code Error")
            return
        }
        
        val editor = e.getData(CommonDataKeys.EDITOR)
        val file = e.getData(CommonDataKeys.VIRTUAL_FILE)
        
        if (editor == null || file == null) {
            Messages.showWarningDialog(project, "Please select a symbol in the editor", "Find Callers")
            return
        }
        
        // Build node ID from file path and selected text (simplified)
        val nodeId = buildNodeId(file.path, editor)
        
        val callers = service.getCallers(nodeId)
        
        if (callers.isEmpty()) {
            Messages.showInfoMessage(project, "No callers found", "Callers")
            return
        }
        
        val message = callers.joinToString("\n") { "${it.from.name} - ${it.from.filePath}" }
        Messages.showInfoMessage(project, message, "Callers")
    }
    
    private fun buildNodeId(filePath: String, editor: com.intellij.openapi.editor.Editor): String {
        // Simplified node ID construction
        // In production, this would use PSI to get the actual symbol
        val relativePath = filePath.substringAfterLast("/")
        return "${relativePath.replace(".", ":")}:_:symbol"
    }
    
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null && e.getData(CommonDataKeys.EDITOR) != null
    }
}
