import React from 'react';

const ReasoningUpdateEvent = ({ data }) => {
  // data is the string content of ReasoningUpdate
  return (
    // Adjusted styling: subtle background, consistent padding/border
    <p className="text-xs italic text-purple-700 dark:text-purple-300 my-1 p-2 border-l-4 border-purple-400 bg-purple-50 dark:bg-gray-700 rounded-r-md">
      {data}
    </p>
  );
};

export default ReasoningUpdateEvent;
