import React from 'react';

// Placeholder component - Integrate based on how progress is signaled via WebSocket events
const ProgressIndicator = ({ progressEvents }) => {
  // Example: Display last 'ReasoningUpdate' or 'ToolExecution' status
  const lastEvent = progressEvents.length > 0 ? progressEvents[progressEvents.length - 1] : null;
  let displayText = 'Idle';

  if (lastEvent) {
    if (lastEvent.ReasoningUpdate) {
      displayText = `Thinking: ${lastEvent.ReasoningUpdate.substring(0, 50)}...`; // Show snippet
    } else if (lastEvent.ToolExecution) {
      const toolName = lastEvent.ToolExecution.details?.tool_name || Object.keys(lastEvent.ToolExecution.details || {})[0] || 'Tool';
      displayText = `Running ${toolName}... (${lastEvent.ToolExecution.status})`;
    } else if (lastEvent.StatusUpdate) {
        const status = lastEvent.StatusUpdate.status;
        let statusText = typeof status === 'string' ? status : Object.keys(status)[0];
         if (typeof status === 'object' && status.Running && status.Running.progress) {
             statusText += `: ${status.Running.progress}`;
         }
        displayText = `Status: ${statusText}`;
    }
     // Add more conditions based on your event types
  }

  return (
    <div className="text-sm text-gray-500 dark:text-gray-400 italic h-6 flex items-center px-4">
      {/* Simple text indicator for now */}
      <span>{displayText}</span>
      {/* Potential spinner */}
      {/* {isLoading && <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-gray-500 ml-2"></div>} */}
    </div>
  );
};

export default ProgressIndicator;
