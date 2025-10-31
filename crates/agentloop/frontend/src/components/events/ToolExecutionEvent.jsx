import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';

// Helper function to parse and format search snippets
const formatSearchSnippets = (snippetsString) => {
  try {
    const snippetsData = JSON.parse(snippetsString);
    if (snippetsData.organic && Array.isArray(snippetsData.organic) && snippetsData.organic.length > 0) {
      return snippetsData.organic.map((item, index) => (
        <div key={index} className="mb-2 p-2 border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800">
          <a href={item.link} target="_blank" rel="noopener noreferrer" className="text-blue-600 dark:text-blue-400 hover:underline font-medium">
            {item.title}
          </a>
          <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">{item.snippet}</p>
          {item.position && <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">Position: {item.position}</p>}
        </div>
      )); // No need for reduce trick if we handle empty array case below
    } else if (snippetsData.organic && Array.isArray(snippetsData.organic) && snippetsData.organic.length === 0) {
       return <p className="text-sm text-gray-600 dark:text-gray-400 italic">No organic results found.</p>;
    }
    // Fallback if structure is different or no organic results field
    console.warn("Search snippets received but structure is unexpected:", snippetsData);
    return <pre className="whitespace-pre-wrap break-words text-gray-700 dark:text-gray-300">{JSON.stringify(snippetsData, null, 2)}</pre>;
  } catch (error) {
    console.error("Failed to parse search snippets:", error);
    // Return the raw string if parsing fails
    return <pre className="whitespace-pre-wrap break-words text-gray-700 dark:text-gray-300">{snippetsString}</pre>;
  }
};


const ToolExecutionEvent = ({ data }) => {
  // data is the ToolResult object from the schema
  const { status, details } = data;
  let toolInfo = '';
  let resultDisplay = null; // Use a variable to hold the JSX for the result
  let bgColor = 'bg-gray-100 dark:bg-gray-700';
  let borderColor = 'border-gray-500';
  let textColor = 'text-gray-800 dark:text-gray-200';

  if (status === 'Success') {
    // Access tool_type and data directly from details (schema: ToolResultDetails::Success)
    // Note: The actual tool_type string constant comes from the schema/backend, e.g., 'SaveContext'
    const { tool_type, data: toolData } = details;
    toolInfo = `${tool_type} (${status})`;
    bgColor = 'bg-green-50 dark:bg-gray-700';
    borderColor = 'border-green-500';
    textColor = 'text-green-800 dark:text-green-300';

    // Specific handling for different tool types based on tool_type enum in schema
    switch (tool_type) {
      case 'Search':
        resultDisplay = (
          <div>
            <p className="font-medium">Query:</p>
            <p className="mb-2 text-gray-700 dark:text-gray-300">{toolData.query}</p>
            <p className="font-medium">Snippets:</p>
            <div className="mt-1">{formatSearchSnippets(toolData.snippets)}</div>
          </div>
        );
        break;
      case 'Browse':
         resultDisplay = (
           <div>
             <p className="font-medium">URL:</p>
             <a href={toolData.url} target="_blank" rel="noopener noreferrer" className="text-blue-600 dark:text-blue-400 hover:underline break-all">{toolData.url}</a>
             <p className="font-medium mt-2">Preview:</p>
             {/* Using ReactMarkdown for potential markdown content */}
             <ReactMarkdown remarkPlugins={[remarkGfm]} className="prose prose-sm dark:prose-invert max-w-none mt-1 text-gray-700 dark:text-gray-300">
                {toolData.content_preview}
             </ReactMarkdown>
           </div>
         );
        break;
      case 'SaveContext': // Handle the SaveContext tool type explicitly
         resultDisplay = (
           <div>
             <p className="font-medium">Context Saved:</p>
             <p className="text-sm text-gray-700 dark:text-gray-300">Source: {toolData.source || 'N/A'}</p>
             <p className="text-sm text-gray-700 dark:text-gray-300">Content Length (Intended): {toolData.content_length}</p>
             {/* Success is implicit from status === 'Success' */}
           </div>
         );
        break;
      case 'Generic':
         resultDisplay = (
           <div>
             <p>Tool Name: {toolData.tool_name}</p>
              <p className="font-medium mt-2">Result:</p>
             <pre className="whitespace-pre-wrap break-words text-gray-700 dark:text-gray-300">{toolData.result}</pre>
           </div>
         );
        break;
      default:
        // Fallback for unknown successful tool types defined in the schema's ToolResultDetails::Success enum
        console.warn("Unknown successful tool type received:", tool_type);
        resultDisplay = <pre className="whitespace-pre-wrap break-words text-gray-700 dark:text-gray-300">{JSON.stringify(toolData, null, 2)}</pre>;
    }

  } else if (status === 'Failure') {
    // Failure details structure: ToolResultDetails::Failure { tool_name: string, error: string }
    toolInfo = `${details.tool_name} (${status})`;
    resultDisplay = <pre className="whitespace-pre-wrap break-words text-gray-700 dark:text-gray-300">{details.error}</pre>;
    bgColor = 'bg-red-50 dark:bg-gray-700';
    borderColor = 'border-red-500';
    textColor = 'text-red-800 dark:text-red-300';
  } else {
    // Handle unexpected status or structure not matching Success/Failure
    console.error("Received ToolExecution with unexpected status or structure:", data);
    toolInfo = `Tool Execution (${status || 'Unknown Status'})`;
    resultDisplay = <pre className="whitespace-pre-wrap break-words text-gray-700 dark:text-gray-300">{JSON.stringify(details, null, 2)}</pre>;
  }

  return (
    <div className={`text-xs p-3 rounded mt-1 mb-2 border-l-4 ${borderColor} ${bgColor} ${textColor} shadow-sm`}>
      <p className="font-semibold mb-1">{toolInfo}</p>
      <div className="pl-2 border-l-2 border-gray-300 dark:border-gray-600">
         {resultDisplay}
      </div>
    </div>
  );
};

export default ToolExecutionEvent;
