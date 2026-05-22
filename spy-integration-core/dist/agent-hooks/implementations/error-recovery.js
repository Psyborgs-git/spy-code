"use strict";
/**
 * Error Recovery Hook Implementation
 * Provides graceful error handling for spy-code operations
 */
Object.defineProperty(exports, "__esModule", { value: true });
exports.ErrorRecoveryHook = void 0;
const types_1 = require("../../types");
class ErrorRecoveryHook {
    constructor(maxRetries = 3, retryDelay = 1000) {
        this.maxRetries = maxRetries;
        this.retryDelay = retryDelay;
    }
}
exports.ErrorRecoveryHook = ErrorRecoveryHook;
{
    return async (context) => {
        if (context.metadata?.error) {
            const error = context.metadata.error;
            // Check if it's a retryable error
            if (this.isRetryableError(error)) {
                const retryCount = context.metadata.retryCount || 0;
                if (retryCount < this.maxRetries) {
                    console.log(`[ErrorRecoveryHook] Retrying operation (attempt ${retryCount + 1}/${this.maxRetries})`);
                    // Update retry count
                    context.metadata.retryCount = retryCount + 1;
                    // Delay before retry
                    await this.delay(this.retryDelay * (retryCount + 1));
                    // Continue to retry
                    return { continue: true };
                }
            }
            // Log the error
            console.error('[ErrorRecoveryHook] Operation failed after retries:', error);
            // Add error metadata
            context.metadata = {
                ...context.metadata,
                errorRecovered: false,
                errorMessage: error.message
            };
        }
        return { continue: true };
    };
}
/**
 * Create a hook handler that recovers from indexing errors
 */
createIndexingErrorHandler();
(context) => Promise;
{
    return async (context) => {
        if (context.metadata?.error && context.metadata?.operation === 'index') {
            console.error('[ErrorRecoveryHook] Indexing failed, but continuing...');
            // Don't block on indexing errors
            context.metadata = {
                ...context.metadata,
                indexingFailed: true,
                errorRecovered: true
            };
        }
        return { continue: true };
    };
}
isRetryableError(error, any);
boolean;
{
    // Retry on connection errors, timeouts, and temporary failures
    if (error instanceof types_1.SpyCodeError) {
        const retryableCodes = ['TIMEOUT', 'CONNECTION_ERROR', 'TEMPORARY_FAILURE'];
        return retryableCodes.includes(error.code);
    }
    // Retry on network errors
    if (error.code === 'ECONNRESET' || error.code === 'ETIMEDOUT') {
        return true;
    }
    return false;
}
delay(ms, number);
Promise < void  > {
    return: new Promise(resolve => setTimeout(resolve, ms))
};
/**
 * Register the error recovery hooks
 */
register(hooks, any);
void {
    hooks, : .registerHook(types_1.HookType.POST_MCP_TOOL_USE, this.createMcpErrorHandler()),
    hooks, : .registerHook(types_1.HookType.POST_WRITE_CODE, this.createIndexingErrorHandler())
};
