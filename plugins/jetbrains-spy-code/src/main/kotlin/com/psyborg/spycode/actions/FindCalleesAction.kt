package com.psyborg.spycode.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.actionSystem.CommonDataKeys
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.psyborg.spycode.services.SpyCodeService

class FindCalleesAction : AnAction() {
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
            Messages.showWarningDialog(project, "Please select a symbol in the editor", "Find Callees")
            return
        }
        
        val nodeId = buildNodeId(file.path, editor)
        
        val callees = service.getCallees(nodeId)
        
        if (callees.isEmpty()) {
            Messages.showInfoMessage(project, "No callees found", "Callees")
            return
        }
        
        val message = callees.joinToString("\n") { "${it.to.name} - ${it.to.filePath}" }
        Messages.showInfoMessage(project, message, "Callees")
    }
    
    private fun buildNodeId(filePath: String, editor: com.intellij.openapi.editor.Editor): String {
        val relativePath = filePath.substringAfterLast("/")
        return "${relativePath.replace(".", ":")}:_:symbol"
    }
    
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null && e.getData(CommonDataKeys.EDITOR) != null
    }
}
