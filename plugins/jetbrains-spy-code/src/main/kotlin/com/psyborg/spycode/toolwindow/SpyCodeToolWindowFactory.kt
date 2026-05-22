package com.psyborg.spycode.toolwindow

import com.intellij.openapi.project.Project
import com.intellij.openapi.wm.ToolWindow
import com.intellij.openapi.wm.ToolWindowFactory
import com.intellij.ui.content.ContentFactory
import com.psyborg.spycode.services.SpyCodeService

class SpyCodeToolWindowFactory : ToolWindowFactory {
    override fun createToolWindowContent(project: Project, toolWindow: ToolWindow) {
        val service = SpyCodeService.getInstance(project)
        val contentFactory = ContentFactory.getInstance()
        
        val spyCodePanel = SpyCodePanel(project, service)
        val content = contentFactory.createContent(spyCodePanel, "", false)
        toolWindow.contentManager.addContent(content)
    }
}
