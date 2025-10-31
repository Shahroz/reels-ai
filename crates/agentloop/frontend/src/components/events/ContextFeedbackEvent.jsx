import React from 'react';

const ContextFeedbackEvent = ({ data }) => {
  // data is the ContextEvaluatorFeedback object
  const { relevance_score, suggestions, needs_update } = data;
  return (
    <div className="text-xs italic text-yellow-700 dark:text-yellow-400 border-l-4 border-yellow-500 pl-2 mt-1 py-1 bg-yellow-50 dark:bg-gray-700">
      <p>Context Feedback (Relevance: {relevance_score.toFixed(2)}) {needs_update ? '[Update Needed]' : ''}</p>
      {suggestions.length > 0 && (
        <ul className="list-disc list-inside ml-2">
          Suggestions: {suggestions.map((s, i) => <li key={i}>{s}</li>)}
        </ul>
      )}
    </div>
  );
};

export default ContextFeedbackEvent;
