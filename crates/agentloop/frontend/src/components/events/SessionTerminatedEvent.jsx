import React from 'react';

const SessionTerminatedEvent = ({ data }) => {
  // data is the SessionTerminated object { session_id, reason }
  return (
    // Adjusted styling: subtle background, consistent padding/border
    <p className="text-xs font-semibold text-red-700 dark:text-red-300 my-1 p-2 border-l-4 border-red-500 bg-red-50 dark:bg-gray-700 rounded-r-md">
      Session Terminated: {data.reason || 'No reason provided.'}
    </p>
  );
};

export default SessionTerminatedEvent;
