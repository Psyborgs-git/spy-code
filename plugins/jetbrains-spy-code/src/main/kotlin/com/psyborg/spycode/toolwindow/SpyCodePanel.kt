package com.psyborg.spycode.toolwindow

import com.intellij.openapi.project.Project
import com.intellij.openapi.ui.SimpleTextAttributes
import com.intellij.ui.JBColor
import com.intellij.ui.components.JBList
import com.intellij.ui.components.JBScrollPane
import com.psyborg.spycode.services.SpyCodeService
import java.awt.BorderLayout
import java.awt.FlowLayout
import javax.swing.*

class SpyCodePanel(private val project: Project, private val service: SpyCodeService) : JPanel() {
    private val searchField = JTextField(30)
    private val searchButton = JButton("Search")
    private val resultsList = JBList<String>()
    private val statsLabel = JLabel()
    
    init {
        layout = BorderLayout()
        
        // Search panel
        val searchPanel = JPanel(FlowLayout())
        searchPanel.add(JLabel("Search:"))
        searchPanel.add(searchField)
        searchPanel.add(searchButton)
        
        add(searchPanel, BorderLayout.NORTH)
        
        // Results panel
        val scrollPane = JBScrollPane(resultsList)
        add(scrollPane, BorderLayout.CENTER)
        
        // Stats panel
        val statsPanel = JPanel(FlowLayout(FlowLayout.LEFT))
        statsPanel.add(statsLabel)
        add(statsPanel, BorderLayout.SOUTH)
        
        // Event listeners
        searchButton.addActionListener {
            performSearch()
        }
        
        resultsList.addListSelectionListener {
            if (!it.valueIsAdjusting) {
                showSelectedResult()
            }
        }
        
        // Load initial stats
        loadStats()
    }
    
    private fun performSearch() {
        val query = searchField.text.trim()
        if (query.isEmpty()) return
        
        val results = service.search(query)
        val listModel = DefaultListModel<String>()
        
        results.forEach { result ->
            listModel.addElement("${result.node.name} - ${result.node.filePath}")
        }
        
        resultsList.model = listModel
    }
    
    private fun showSelectedResult() {
        val selected = resultsList.selectedValue ?: return
        // Show node details (simplified)
        JOptionPane.showMessageDialog(this, selected, "Node Details", JOptionPane.INFORMATION_MESSAGE)
    }
    
    private fun loadStats() {
        val stats = service.getStats()
        statsLabel.text = "Nodes: ${stats.nodeCount} | Edges: ${stats.edgeCount} | Files: ${stats.fileCount}"
    }
}
