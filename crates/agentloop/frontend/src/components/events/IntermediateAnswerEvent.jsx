import React from 'react';import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';


/**
 * Displays an intermediate answer event.
 * @param {{ data: string }} props - Contains the intermediate answer string.
 */
const IntermediateAnswerEvent = ({ data }) => {
  return (
    // Adjusted styling for consistency: less padding, subtle background
    <div className="text-sm text-gray-700 dark:text-gray-300 my-1 p-2 border-l-4 border-blue-400 bg-blue-50 dark:bg-gray-700 rounded-r-md">
      <p className="font-medium">Intermediate Answer:</p>
      <div className="prose prose-sm dark:prose-invert max-w-none break-words">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>{data}</ReactMarkdown>
      </div>

    </div>
  );
};

export default IntermediateAnswerEvent;
