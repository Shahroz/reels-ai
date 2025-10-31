import React from 'react';

const StatusUpdateEvent = ({ data }) => {
  // data is the StatusResponse object
  const { status } = data;
  let statusText = '';

  if (typeof status === 'string') {
    statusText = status;
  } else if (typeof status === 'object' && status !== null) {
    const statusKey = Object.keys(status)[0];
    statusText = statusKey;
    if (status[statusKey] && status[statusKey].progress) {
      statusText += `: ${status[statusKey].progress}`;
    }
  } else {
     statusText = 'Unknown Status';
  }

  return (
    // Adjusted styling: subtle background, consistent padding/border
    <p className="text-xs italic text-gray-600 dark:text-gray-400 my-1 p-1 border-l-4 border-gray-400 bg-gray-50 dark:bg-gray-700 rounded-r-md">
      Status: {statusText}
    </p>
  );
};

export default StatusUpdateEvent;
