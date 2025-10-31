import React from 'react';

// Import the event components
import ReasoningUpdateEvent from './events/ReasoningUpdateEvent';
import IntermediateAnswerEvent from './events/IntermediateAnswerEvent';
import ToolExecutionEvent from './events/ToolExecutionEvent';
import ContextFeedbackEvent from './events/ContextFeedbackEvent';
import StatusUpdateEvent from './events/StatusUpdateEvent';
import FinalAnswerEvent from './events/FinalAnswerEvent';
import SessionTerminatedEvent from './events/SessionTerminatedEvent';

// Helper function to render complex event data nicely using specific components
const renderEventData = (data) => {
  if (!data) return null;

  // Render specific component based on the event type key
  if (data.ReasoningUpdate) {
    return <ReasoningUpdateEvent data={data.ReasoningUpdate} />;
  }
  if (data.ToolExecution) {
    return <ToolExecutionEvent data={data.ToolExecution} />;
  }
  if (data.ContextFeedback) {
    return <ContextFeedbackEvent data={data.ContextFeedback} />;
  }
  if (data.StatusUpdate) {
    return <StatusUpdateEvent data={data.StatusUpdate} />;
  }
  // New unified AgentAnswer event
  if (data.AgentAnswer) {
    const { content, is_final } = data.AgentAnswer;
    if (is_final) {
      return <FinalAnswerEvent data={{ content }} />;
    } else {
      return <IntermediateAnswerEvent data={content} />;
    }
  }
  if (data.IntermediateAnswer) {
    return <IntermediateAnswerEvent data={data.IntermediateAnswer} />;
  }
  if (data.FinalAnswer) {
    return <FinalAnswerEvent data={data.FinalAnswer} />;
  }
  if (data.SessionTerminated) {
    return <SessionTerminatedEvent data={data.SessionTerminated} />;
  }

  // Fallback for unknown or raw events
  console.warn("Unknown event type received:", data);
  return <pre className="text-xs whitespace-pre-wrap break-words bg-gray-100 dark:bg-gray-700 p-2 rounded border border-dashed border-gray-400">Unknown Event: {JSON.stringify(data, null, 2)}</pre>;
};


const MessageBubble = ({ message }) => {
  const { type, text, data } = message; // 'user', 'agent', 'system', 'error', 'event'

  let bubbleClasses = 'inline-block px-4 py-2 shadow rounded-lg max-w-xl break-words '; // Common classes
  let wrapperClasses = 'flex mb-3 ';
  let content = text; // Default content is text

  switch (type) {
    case 'user':
      // Mimic style_grok user message: white background, rounded-full (using rounded-lg for better fit)
      bubbleClasses += 'bg-white dark:bg-gray-700 text-gray-700 dark:text-gray-200';
      wrapperClasses += 'justify-end';
      break;
    case 'agent': // Generic agent message - use a slightly different style
      bubbleClasses += 'bg-indigo-500 text-white';
      wrapperClasses += 'justify-start';
      break;
    case 'system':
       // Use a less prominent style for system messages
      bubbleClasses = 'text-center w-full max-w-none text-xs text-gray-500 dark:text-gray-400 italic';
      wrapperClasses += 'justify-center'; // Center system messages
      break;
    case 'error':
      bubbleClasses += 'bg-red-100 dark:bg-red-900 border border-red-400 text-red-700 dark:text-red-300 px-4 py-2'; // More distinct error style
      wrapperClasses += 'justify-start';
       bubbleClasses = bubbleClasses.replace('shadow', ''); // Remove shadow for error
      break;
    case 'event':
      // Events use their own components which define styling. Wrapper provides alignment.
       wrapperClasses += 'justify-start w-full'; // Events might need full width
       content = renderEventData(data);
       // Render content directly without extra bubble div if components handle all styling
       return <div className={wrapperClasses}>{content}</div>;
    default: // Fallback style
      bubbleClasses += 'bg-gray-200 dark:bg-gray-600 text-gray-800 dark:text-gray-200';
      wrapperClasses += 'justify-start';
  }

  return (
    <div className={wrapperClasses}>
      <div className={bubbleClasses}>
        {content}
      </div>
    </div>
  );
};

export default MessageBubble;
