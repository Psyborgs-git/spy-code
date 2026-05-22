"use strict";
/**
 * Event Bus - Pub/sub system for IDE events
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.eventBus = exports.EventBus = exports.EventType = void 0;
var EventType;
(function (EventType) {
    // File events
    EventType["FILE_OPENED"] = "file_opened";
    EventType["FILE_CLOSED"] = "file_closed";
    EventType["FILE_SAVED"] = "file_saved";
    EventType["FILE_CHANGED"] = "file_changed";
    // Cursor events
    EventType["CURSOR_MOVED"] = "cursor_moved";
    EventType["SELECTION_CHANGED"] = "selection_changed";
    // Index events
    EventType["INDEX_STARTED"] = "index_started";
    EventType["INDEX_COMPLETED"] = "index_completed";
    EventType["INDEX_FAILED"] = "index_failed";
    // Search events
    EventType["SEARCH_STARTED"] = "search_started";
    EventType["SEARCH_COMPLETED"] = "search_completed";
    EventType["SEARCH_FAILED"] = "search_failed";
    // Navigation events
    EventType["NAVIGATION_REQUESTED"] = "navigation_requested";
    EventType["NAVIGATION_COMPLETED"] = "navigation_completed";
    // Agent events
    EventType["AGENT_STARTED"] = "agent_started";
    EventType["AGENT_COMPLETED"] = "agent_completed";
    EventType["AGENT_ERROR"] = "agent_error";
    // Hook events
    EventType["HOOK_REGISTERED"] = "hook_registered";
    EventType["HOOK_UNREGISTERED"] = "hook_unregistered";
    EventType["HOOK_EXECUTED"] = "hook_executed";
    EventType["HOOK_ERROR"] = "hook_error";
    // MCP events
    EventType["MCP_CONNECTED"] = "mcp_connected";
    EventType["MCP_DISCONNECTED"] = "mcp_disconnected";
    EventType["MCP_ERROR"] = "mcp_error";
    // Cache events
    EventType["CACHE_HIT"] = "cache_hit";
    EventType["CACHE_MISS"] = "cache_miss";
    EventType["CACHE_CLEARED"] = "cache_cleared";
    // UI events
    EventType["PANEL_OPENED"] = "panel_opened";
    EventType["PANEL_CLOSED"] = "panel_closed";
    EventType["NOTIFICATION_SHOWN"] = "notification_shOWN";
})(EventType || (exports.EventType = EventType = {}));
class EventBus {
    constructor() {
        this.listeners = new Map();
        this.eventHistory = new Map();
        this.maxHistorySize = 100;
        this.setupEventHistory();
    }
    static getInstance() {
        if (!EventBus.instance) {
            EventBus.instance = new EventBus();
        }
        return EventBus.instance;
    }
    /**
     * Setup event history tracking
     */
    setupEventHistory() {
        for (const eventType of Object.values(EventType)) {
            this.eventHistory.set(eventType, []);
        }
    }
    /**
     * Emit an event
     */
    emit(event, payload) {
        // Add to history
        const history = this.eventHistory.get(event);
        if (history) {
            history.push({
                timestamp: Date.now(),
                payload
            });
            // Trim history if needed
            if (history.length > this.maxHistorySize) {
                history.shift();
            }
        }
        // Notify listeners
        const eventListeners = this.listeners.get(event);
        if (eventListeners) {
            for (const listener of eventListeners) {
                try {
                    listener(payload);
                }
                catch (error) {
                    console.error(`Error in event listener for ${event}:`, error);
                }
            }
        }
        return true;
    }
    /**
     * Subscribe to an event
     */
    on(event, listener) {
        if (!this.listeners.has(event)) {
            this.listeners.set(event, []);
        }
        this.listeners.get(event).push(listener);
        return this;
    }
    /**
     * Subscribe to an event once
     */
    once(event, listener) {
        const onceWrapper = (payload) => {
            listener(payload);
            this.off(event, onceWrapper);
        };
        return this.on(event, onceWrapper);
    }
    /**
     * Unsubscribe from an event
     */
    off(event, listener) {
        const eventListeners = this.listeners.get(event);
        if (eventListeners) {
            const index = eventListeners.indexOf(listener);
            if (index > -1) {
                eventListeners.splice(index, 1);
            }
        }
        return this;
    }
    /**
     * Get event history
     */
    getHistory(eventType) {
        return [...(this.eventHistory.get(eventType) || [])];
    }
    /**
     * Get all event history
     */
    getAllHistory() {
        const result = new Map();
        for (const [eventType, history] of this.eventHistory.entries()) {
            result.set(eventType, [...history]);
        }
        return result;
    }
    /**
     * Clear event history
     */
    clearHistory(eventType) {
        if (eventType) {
            this.eventHistory.set(eventType, []);
        }
        else {
            for (const eventType of Object.values(EventType)) {
                this.eventHistory.set(eventType, []);
            }
        }
    }
    /**
     * Get event statistics
     */
    getStats() {
        let totalEvents = 0;
        const eventsByType = new Map();
        const listenersByType = new Map();
        for (const [eventType, history] of this.eventHistory.entries()) {
            const count = history.length;
            totalEvents += count;
            eventsByType.set(eventType, count);
            listenersByType.set(eventType, (this.listeners.get(eventType) || []).length);
        }
        return {
            totalEvents,
            eventsByType,
            listenersByType
        };
    }
    /**
     * Clear all listeners
     */
    clearAllListeners() {
        this.listeners.clear();
    }
    /**
     * Update max history size
     */
    updateMaxHistorySize(size) {
        this.maxHistorySize = size;
        // Trim existing history
        for (const history of this.eventHistory.values()) {
            while (history.length > this.maxHistorySize) {
                history.shift();
            }
        }
    }
}
exports.EventBus = EventBus;
// Export singleton instance
exports.eventBus = EventBus.getInstance();
