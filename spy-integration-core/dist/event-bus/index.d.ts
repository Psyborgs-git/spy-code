/**
 * Event Bus - Pub/sub system for IDE events
 */
export declare enum EventType {
    FILE_OPENED = "file_opened",
    FILE_CLOSED = "file_closed",
    FILE_SAVED = "file_saved",
    FILE_CHANGED = "file_changed",
    CURSOR_MOVED = "cursor_moved",
    SELECTION_CHANGED = "selection_changed",
    INDEX_STARTED = "index_started",
    INDEX_COMPLETED = "index_completed",
    INDEX_FAILED = "index_failed",
    SEARCH_STARTED = "search_started",
    SEARCH_COMPLETED = "search_completed",
    SEARCH_FAILED = "search_failed",
    NAVIGATION_REQUESTED = "navigation_requested",
    NAVIGATION_COMPLETED = "navigation_completed",
    AGENT_STARTED = "agent_started",
    AGENT_COMPLETED = "agent_completed",
    AGENT_ERROR = "agent_error",
    HOOK_REGISTERED = "hook_registered",
    HOOK_UNREGISTERED = "hook_unregistered",
    HOOK_EXECUTED = "hook_executed",
    HOOK_ERROR = "hook_error",
    MCP_CONNECTED = "mcp_connected",
    MCP_DISCONNECTED = "mcp_disconnected",
    MCP_ERROR = "mcp_error",
    CACHE_HIT = "cache_hit",
    CACHE_MISS = "cache_miss",
    CACHE_CLEARED = "cache_cleared",
    PANEL_OPENED = "panel_opened",
    PANEL_CLOSED = "panel_closed",
    NOTIFICATION_SHOWN = "notification_shOWN"
}
export interface EventPayload {
    [EventType.FILE_OPENED]: {
        filePath: string;
        language: string;
    };
    [EventType.FILE_CLOSED]: {
        filePath: string;
    };
    [EventType.FILE_SAVED]: {
        filePath: string;
        content: string;
    };
    [EventType.FILE_CHANGED]: {
        filePath: string;
        changes: any;
    };
    [EventType.CURSOR_MOVED]: {
        filePath: string;
        line: number;
        character: number;
    };
    [EventType.SELECTION_CHANGED]: {
        filePath: string;
        selection: any;
    };
    [EventType.INDEX_STARTED]: {
        full: boolean;
    };
    [EventType.INDEX_COMPLETED]: {
        stats: any;
    };
    [EventType.INDEX_FAILED]: {
        error: Error;
    };
    [EventType.SEARCH_STARTED]: {
        query: string;
        options: any;
    };
    [EventType.SEARCH_COMPLETED]: {
        query: string;
        results: any;
    };
    [EventType.SEARCH_FAILED]: {
        query: string;
        error: Error;
    };
    [EventType.NAVIGATION_REQUESTED]: {
        target: string;
        type: string;
    };
    [EventType.NAVIGATION_COMPLETED]: {
        target: string;
        success: boolean;
    };
    [EventType.AGENT_STARTED]: {
        agentId: string;
        task: string;
    };
    [EventType.AGENT_COMPLETED]: {
        agentId: string;
        result: any;
    };
    [EventType.AGENT_ERROR]: {
        agentId: string;
        error: Error;
    };
    [EventType.HOOK_REGISTERED]: {
        hookType: string;
        handler: any;
    };
    [EventType.HOOK_UNREGISTERED]: {
        hookType: string;
        handler: any;
    };
    [EventType.HOOK_EXECUTED]: {
        hookType: string;
        handler: any;
        result: any;
    };
    [EventType.HOOK_ERROR]: {
        hookType: string;
        handler: any;
        error: Error;
    };
    [EventType.MCP_CONNECTED]: Record<string, never>;
    [EventType.MCP_DISCONNECTED]: {
        code: number | null;
    };
    [EventType.MCP_ERROR]: {
        error: Error;
    };
    [EventType.CACHE_HIT]: {
        key: string;
    };
    [EventType.CACHE_MISS]: {
        key: string;
    };
    [EventType.CACHE_CLEARED]: Record<string, never>;
    [EventType.PANEL_OPENED]: {
        panelId: string;
    };
    [EventType.PANEL_CLOSED]: {
        panelId: string;
    };
    [EventType.NOTIFICATION_SHOWN]: {
        message: string;
        type: string;
    };
}
export declare class EventBus {
    private static instance;
    private listeners;
    private eventHistory;
    private maxHistorySize;
    private constructor();
    static getInstance(): EventBus;
    /**
     * Setup event history tracking
     */
    private setupEventHistory;
    /**
     * Emit an event
     */
    emit(event: EventType, payload: any): boolean;
    /**
     * Subscribe to an event
     */
    on(event: EventType, listener: (payload: any) => void): this;
    /**
     * Subscribe to an event once
     */
    once(event: EventType, listener: (payload: any) => void): this;
    /**
     * Unsubscribe from an event
     */
    off(event: EventType, listener: (payload: any) => void): this;
    /**
     * Get event history
     */
    getHistory(eventType: EventType): Array<{
        timestamp: number;
        payload: any;
    }>;
    /**
     * Get all event history
     */
    getAllHistory(): Map<EventType, Array<{
        timestamp: number;
        payload: any;
    }>>;
    /**
     * Clear event history
     */
    clearHistory(eventType?: EventType): void;
    /**
     * Get event statistics
     */
    getStats(): {
        totalEvents: number;
        eventsByType: Map<EventType, number>;
        listenersByType: Map<EventType, number>;
    };
    /**
     * Clear all listeners
     */
    clearAllListeners(): void;
    /**
     * Update max history size
     */
    updateMaxHistorySize(size: number): void;
}
export declare const eventBus: EventBus;
