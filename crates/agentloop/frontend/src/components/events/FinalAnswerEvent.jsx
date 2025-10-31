import React, { useState } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm'; // For GitHub Flavored Markdown (tables, etc.)
import { ClipboardCopyIcon } from '@heroicons/react/outline'; // Example icon import

const FinalAnswerEvent = ({ data }) => {
  // Expect data to be { title: string, content: string }
  const { title, content } = data || { title: 'Final Answer', content: '' }; // Provide defaults
  const [copyStatus, setCopyStatus] = useState('Copy'); // 'Copy', 'Copied!'

  const handleCopy = async () => {
    if (!navigator.clipboard) {
      // Fallback for older browsers or insecure contexts
      console.error("Clipboard API not available.");
      setCopyStatus('Error');
      setTimeout(() => setCopyStatus('Copy'), 2000);
      return;
    }
    try {
      await navigator.clipboard.writeText(content); // Copy the content part
      setCopyStatus('Copied!');
      setTimeout(() => setCopyStatus('Copy'), 2000); // Reset after 2 seconds
    } catch (err) {
      console.error('Failed to copy text: ', err);
      setCopyStatus('Error');
      setTimeout(() => setCopyStatus('Copy'), 2000);
    }
  };

  return (
    // Adjusted styling: subtle background, consistent padding/border
    <div className="text-sm text-green-800 dark:text-green-300 my-1 p-3 border-l-4 border-green-500 bg-green-50 dark:bg-gray-750 rounded-r-md relative group">
      <div className="flex justify-between items-start mb-2"> {/* Increased mb */}
        <p className="font-semibold">Final Answer:</p>
        <button
          onClick={handleCopy}
          className={`absolute top-1 right-1 p-1 rounded ${
            copyStatus === 'Copied!'
              ? 'bg-green-200 dark:bg-green-700 text-green-800 dark:text-green-200'
              : 'bg-gray-200 dark:bg-gray-600 text-gray-600 dark:text-gray-300 hover:bg-gray-300 dark:hover:bg-gray-500'
          } opacity-0 group-hover:opacity-100 transition-opacity text-xs flex items-center`}
          aria-label="Copy final answer"
        >
          <ClipboardCopyIcon className="h-4 w-4 mr-1" />
          {copyStatus}
        </button>
      </div>
      {title && <p className="font-medium text-base mb-2">{title}</p>} {/* Display title */}
      {/* Render content as Markdown */}
      {/* Added prose styles for better markdown rendering */}
      <div className="prose prose-sm dark:prose-invert max-w-none break-words">
        <ReactMarkdown remarkPlugins={[remarkGfm]}>{content}</ReactMarkdown> {/* Render content */}
      </div>
    </div>
  );
};

export default FinalAnswerEvent;