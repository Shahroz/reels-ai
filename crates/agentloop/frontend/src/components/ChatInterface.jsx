import React, { useRef, useEffect, useState } from 'react';
import MessageBubble from './MessageBubble';
import InputArea from './InputArea';
import SelectResearchItemsModal from './SelectResearchItemsModal';

// Added onTerminate and sessionId props
const ChatInterface = ({ messages, onSendMessage, onTerminate, sessionId, isConnected, isProcessing }) => {
  const messagesEndRef = useRef(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(scrollToBottom, [messages]); // Scroll on new messages

  const [showResearchModal, setShowResearchModal] = useState(false);
  const [selectedResearchItems, setSelectedResearchItems] = useState([]);

  const handleToggleResearchModal = () => {
    setShowResearchModal(!showResearchModal);
  };

  const handleResearchItemsSelected = (itemIds) => {
    setSelectedResearchItems(itemIds);
    setShowResearchModal(false);
  };

  const handleSendMessageWithAttachment = (messageText) => {
    let finalMessage = messageText;
    if (selectedResearchItems.length > 0) {
      finalMessage += `\n\nAttached context: [${selectedResearchItems.join(', ')}]`;
      setSelectedResearchItems([]); // Clear after preparing
    }
    onSendMessage(finalMessage); // Call the original onSendMessage prop
  };

  // Calculate disabled state for InputArea
  const isDisabled = isProcessing && !!sessionId;
  // Calculate connecting state for spinner display
  const isConnecting = !isConnected && !!sessionId;
  return (
    // Removed outer container styling (bg, shadow, border, max-width) - App.jsx handles layout
    // Use h-full and w-full to fill the space provided by App's main section
    <div className="flex flex-col h-full w-full max-w-4xl mx-auto">
      {/* Optional: Header within the chat window (moved from style_grok header) */}
       {sessionId && (
         <div className="p-2 border-b border-gray-200 dark:border-gray-700 flex justify-between items-center bg-white dark:bg-gray-800 rounded-t-lg">
           <span className="text-xs text-gray-500 dark:text-gray-400">Session Active (ID: ...{sessionId.slice(-6)})</span>
           <button
             onClick={onTerminate}
             className="bg-red-500 hover:bg-red-700 text-white font-bold py-1 px-2 rounded text-xs"
           >
             Terminate
           </button>
         </div>
       )}

      {/* Message List Area */}
      {/* Added background and rounded corners (bottom only if header exists) */}
      <div className={`flex-1 overflow-y-auto space-y-4 p-4 custom-scrollbar bg-white dark:bg-gray-800 ${sessionId ? '' : 'rounded-t-lg'}`}>
        {/* Placeholder for initial message if needed */}
        {messages.length === 0 && !isConnecting && !isProcessing && (
            <div className="text-center text-sm text-gray-500 mt-4">
                {sessionId ? "Waiting for agent..." : "Enter an instruction below to start."}
            </div>
        )}
        {messages.map((msg, index) => (
          <MessageBubble key={index} message={msg} />
        ))}
        {isConnecting && (
          <div className="flex justify-center items-center p-4">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-indigo-500"></div>
            <span className="ml-2 text-sm text-gray-500 dark:text-gray-400">Connecting...</span>
          </div>
        )}
        {isProcessing && isConnected && (
          <div className="flex justify-center items-center p-4">
            <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-500"></div>
            <span className="ml-2 text-sm text-gray-500 dark:text-gray-400">Processing...</span>
          </div>
        )}
        <div ref={messagesEndRef} /> {/* Anchor for scrolling */}
      </div>

      {/* Input Area - Now acts as the footer */}
      <InputArea onSendMessage={handleSendMessageWithAttachment} disabled={isDisabled} onAttachClick={handleToggleResearchModal} />

      {showResearchModal && (
        <SelectResearchItemsModal
          onClose={handleToggleResearchModal}
          onItemsSelected={handleResearchItemsSelected}
        />
      )}
    </div>
  );
};

// Custom scrollbar styling (keep as is)
const styles = `
.custom-scrollbar::-webkit-scrollbar {
  width: 6px; /* Thinner scrollbar */
}
.custom-scrollbar::-webkit-scrollbar-track {
  background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background-color: rgba(156, 163, 175, 0.4); /* Lighter gray with more transparency */
  border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background-color: rgba(107, 114, 128, 0.6); /* Slightly darker on hover */
}
/* For Firefox */
.custom-scrollbar {
  scrollbar-width: thin;
  scrollbar-color: rgba(156, 163, 175, 0.4) transparent;
}
`
const styleSheet = document.getElementById('custom-scrollbar-styles');
if (!styleSheet) {
  const newStyleSheet = document.createElement("style");
  newStyleSheet.id = 'custom-scrollbar-styles';
  newStyleSheet.innerText = styles;
  document.head.appendChild(newStyleSheet);
}


export default ChatInterface;
