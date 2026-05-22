package com.psyborg.spycode.actions

import com.intellij.openapi.actionSystem.AnAction
import com.intellij.openapi.actionSystem.AnActionEvent
import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.Messages
import com.intellij.openapi.progress.ProgressIndicator
import com.intellij.openapi.progress.ProgressManager
import com.intellij.openapi.progress.Task
import com.psyborg.spycode.services.SpyCodeService

class IndexAction : AnAction() {
    override fun actionPerformed(e: AnActionEvent) {
        val project = e.project ?: return
        val service = SpyCodeService.getInstance(project)
        
        if (!service.isAvailable()) {
            Messages.showErrorDialog(project, "Spy-Code is not installed or not available", "Spy-Code Error")
            return
        }
        
        ProgressManager.getInstance().run(object : Task.Backgroundable(project, "Indexing Codebase", true) {
            override fun run(indicator: ProgressIndicator) {
                indicator.text = "Indexing codebase with Spy-Code..."
                service.index()
            }
            
            override fun onSuccess() {
                Messages.showInfoMessage(project, "Codebase indexed successfully", "Spy-Code")
            }
            
            override fun onThrowable(error: Throwable) {
                Messages.showErrorDialog(project, "Indexing failed: ${error.message}", "Spy-Code Error")
            }
        })
    }
    
    override fun update(e: AnActionEvent) {
        e.presentation.isEnabled = e.project != null
    }
}
