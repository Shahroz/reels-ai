import React, { useState } from 'react';

const InputArea = ({ onSendMessage, disabled }) => {
  const [message, setMessage] = useState('');

  const handleSend = () => {
    if (message.trim() && !disabled) {
      onSendMessage(message);
      setMessage(''); // Clear input after sending
    }
  };

  const handleKeyPress = (event) => {
    if (event.key === 'Enter' && !event.shiftKey) {
      event.preventDefault(); // Prevent newline on Enter
      handleSend();
    }
  };

  return (
    // Mimic footer structure from style_grok.html
    <div className="w-full border-t border-gray-200 dark:border-gray-700 p-4 bg-white dark:bg-gray-800 rounded-b-lg">
      <div className="flex items-center gap-2">
        {/* Input field styled like style_grok */}
        <input
          type="text"
          value={message}
          onChange={(e) => setMessage(e.target.value)}
          onKeyPress={handleKeyPress}
          placeholder={disabled ? "Waiting for response or connection..." : "How can Loupe help?"}
          className="flex-grow p-3 rounded-full border border-gray-300 focus:outline-none focus:ring-2 focus:ring-indigo-400 bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-gray-200"
          disabled={disabled}
        />
        {/* Optional: Add buttons like style_grok if needed, or just the send button */}
        {/* Example Send button matching style_grok button style */}
        <button
          onClick={handleSend}
          className="px-4 py-2 rounded-full bg-indigo-500 hover:bg-indigo-600 text-white text-sm font-semibold disabled:opacity-50 disabled:cursor-not-allowed"
          disabled={disabled || !message.trim()}
        >
          Send
        </button>
         {/* Example placeholder buttons from style_grok - non-functional */}
        {/* <button className="px-4 py-2 rounded-full border border-gray-300 bg-white text-gray-700 text-sm">DeepSearch</button>
        <button className="px-4 py-2 rounded-full border border-gray-300 bg-white text-gray-700 text-sm">Think</button>
        <button className="px-4 py-2 rounded-full bg-gray-100 text-gray-700 text-sm">Loupe 3</button> */}
      </div>
    </div>
  );
};

export default InputArea;
