/**
 * Event Bus - Pub/sub system for IDE events
 */

export enum EventType {
  // File events
  FILE_OPENED = 'file_opened',
  FILE_CLOSED = 'file_closed',
  FILE_SAVED = 'file_saved',
  FILE_CHANGED = 'file_changed',

  // Cursor events
  CURSOR_MOVED = 'cursor_moved',
  SELECTION_CHANGED = 'selection_changed',

  // Index events
  INDEX_STARTED = 'index_started',
  INDEX_COMPLETED = 'index_completed',
  INDEX_FAILED = 'index_failed',

  // Search events
  SEARCH_STARTED = 'search_started',
  SEARCH_COMPLETED = 'search_completed',
  SEARCH_FAILED = 'search_failed',

  // Navigation events
  NAVIGATION_REQUESTED = 'navigation_requested',
  NAVIGATION_COMPLETED = 'navigation_completed',

  // Agent events
  AGENT_STARTED = 'agent_started',
  AGENT_COMPLETED = 'agent_completed',
  AGENT_ERROR = 'agent_error',

  // Hook events
  HOOK_REGISTERED = 'hook_registered',
  HOOK_UNREGISTERED = 'hook_unregistered',
  HOOK_EXECUTED = 'hook_executed',
  HOOK_ERROR = 'hook_error',

  // MCP events
  MCP_CONNECTED = 'mcp_connected',
  MCP_DISCONNECTED = 'mcp_disconnected',
  MCP_ERROR = 'mcp_error',

  // Cache events
  CACHE_HIT = 'cache_hit',
  CACHE_MISS = 'cache_miss',
  CACHE_CLEARED = 'cache_cleared',

  // UI events
  PANEL_OPENED = 'panel_opened',
  PANEL_CLOSED = 'panel_closed',
  NOTIFICATION_SHOWN = 'notification_shOWN',
}

export interface EventPayload {
  [EventType.FILE_OPENED]: { filePath: string; language: string };
  [EventType.FILE_CLOSED]: { filePath: string };
  [EventType.FILE_SAVED]: { filePath: string; content: string };
  [EventType.FILE_CHANGED]: { filePath: string; changes: any };

  [EventType.CURSOR_MOVED]: { filePath: string; line: number; character: number };
  [EventType.SELECTION_CHANGED]: { filePath: string; selection: any };

  [EventType.INDEX_STARTED]: { full: boolean };
  [EventType.INDEX_COMPLETED]: { stats: any };
  [EventType.INDEX_FAILED]: { error: Error };

  [EventType.SEARCH_STARTED]: { query: string; options: any };
  [EventType.SEARCH_COMPLETED]: { query: string; results: any };
  [EventType.SEARCH_FAILED]: { query: string; error: Error };

  [EventType.NAVIGATION_REQUESTED]: { target: string; type: string };
  [EventType.NAVIGATION_COMPLETED]: { target: string; success: boolean };

  [EventType.AGENT_STARTED]: { agentId: string; task: string };
  [EventType.AGENT_COMPLETED]: { agentId: string; result: any };
  [EventType.AGENT_ERROR]: { agentId: string; error: Error };

  [EventType.HOOK_REGISTERED]: { hookType: string; handler: any };
  [EventType.HOOK_UNREGISTERED]: { hookType: string; handler: any };
  [EventType.HOOK_EXECUTED]: { hookType: string; handler: any; result: any };
  [EventType.HOOK_ERROR]: { hookType: string; handler: any; error: Error };

  [EventType.MCP_CONNECTED]: Record<string, never>;
  [EventType.MCP_DISCONNECTED]: { code: number | null };
  [EventType.MCP_ERROR]: { error: Error };

  [EventType.CACHE_HIT]: { key: string };
  [EventType.CACHE_MISS]: { key: string };
  [EventType.CACHE_CLEARED]: Record<string, never>;

  [EventType.PANEL_OPENED]: { panelId: string };
  [EventType.PANEL_CLOSED]: { panelId: string };
  [EventType.NOTIFICATION_SHOWN]: { message: string; type: string };
}

export class EventBus {
  private static instance: EventBus;
  private listeners: Map<EventType, Array<(payload: any) => void>> = new Map();
  private eventHistory: Map<EventType, Array<{ timestamp: number; payload: any }>> = new Map();
  private maxHistorySize: number = 100;

  private constructor() {
    this.setupEventHistory();
  }

  static getInstance(): EventBus {
    if (!EventBus.instance) {
      EventBus.instance = new EventBus();
    }
    return EventBus.instance;
  }

  /**
   * Setup event history tracking
   */
  private setupEventHistory(): void {
    for (const eventType of Object.values(EventType)) {
      this.eventHistory.set(eventType as EventType, []);
    }
  }

  /**
   * Emit an event
   */
  emit(event: EventType, payload: any): boolean {
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
        } catch (error) {
          console.error(`Error in event listener for ${event}:`, error);
        }
      }
    }

    return true;
  }

  /**
   * Subscribe to an event
   */
  on(event: EventType, listener: (payload: any) => void): this {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, []);
    }
    this.listeners.get(event)!.push(listener);
    return this;
  }

  /**
   * Subscribe to an event once
   */
  once(event: EventType, listener: (payload: any) => void): this {
    const onceWrapper = (payload: any) => {
      listener(payload);
      this.off(event, onceWrapper);
    };
    return this.on(event, onceWrapper);
  }

  /**
   * Unsubscribe from an event
   */
  off(event: EventType, listener: (payload: any) => void): this {
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
  getHistory(eventType: EventType): Array<{ timestamp: number; payload: any }> {
    return [...(this.eventHistory.get(eventType) || [])];
  }

  /**
   * Get all event history
   */
  getAllHistory(): Map<EventType, Array<{ timestamp: number; payload: any }>> {
    const result = new Map();
    for (const [eventType, history] of this.eventHistory.entries()) {
      result.set(eventType, [...history]);
    }
    return result;
  }

  /**
   * Clear event history
   */
  clearHistory(eventType?: EventType): void {
    if (eventType) {
      this.eventHistory.set(eventType, []);
    } else {
      for (const eventType of Object.values(EventType)) {
        this.eventHistory.set(eventType as EventType, []);
      }
    }
  }

  /**
   * Get event statistics
   */
  getStats(): {
    totalEvents: number;
    eventsByType: Map<EventType, number>;
    listenersByType: Map<EventType, number>;
  } {
    let totalEvents = 0;
    const eventsByType = new Map<EventType, number>();
    const listenersByType = new Map<EventType, number>();

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
  clearAllListeners(): void {
    this.listeners.clear();
  }

  /**
   * Update max history size
   */
  updateMaxHistorySize(size: number): void {
    this.maxHistorySize = size;

    // Trim existing history
    for (const history of this.eventHistory.values()) {
      while (history.length > this.maxHistorySize) {
        history.shift();
      }
    }
  }
}

// Export singleton instance
export const eventBus = EventBus.getInstance();
