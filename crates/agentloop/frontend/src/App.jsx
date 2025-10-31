import React, { useState, useEffect, useCallback } from 'react';
import useAgentLoopWebSocket from './hooks/useAgentLoopWebSocket';
import ChatInterface from './components/ChatInterface';

function App() {
  const [serverAddress, setServerAddress] = useState('ws://localhost:8080/ws'); // Keep for WebSocket hook
  const [sessionId, setSessionId] = useState(null);
  const [isProcessing, setIsProcessing] = useState(false); // Track if backend is busy
  const [messages, setMessages] = useState([]);

  const { ws, isConnected, connectWebSocket, disconnectWebSocket, sendMessage: sendWsMessage } = useAgentLoopWebSocket(
    serverAddress,
    (event) => {
      console.log('Received event:', event);
      setMessages((prevMessages) => [...prevMessages, { type: 'event', data: event }]);
      // Check for AgentAnswer event to unlock input when final
      if (event && event.AgentAnswer) {
        const { is_final } = event.AgentAnswer;
        if (is_final) {
          console.log('Detected AgentAnswer final, enabling input.');
          setIsProcessing(false);
        }
      }
    }
  );

  const initializeSession = useCallback(async (instruction) => {
    if (!instruction || sessionId) return;
    console.log('Initializing session with instruction:', instruction);
    setMessages([{ type: 'user', text: instruction }]);
    setIsProcessing(true); // Start processing
    try {
      const response = await fetch('http://localhost:8080/research', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ instruction }),
      });
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      const data = await response.json();
      console.log('Research started, Session ID:', data.session_id);
      setSessionId(data.session_id);

      const urlObject = new URL(serverAddress.replace(/^ws/, 'http'));
      const wsProtocol = serverAddress.startsWith('wss') ? 'wss:' : 'ws:';
      const wsUrl = `${wsProtocol}//${urlObject.host}/session/${data.session_id}/stream`;
      console.log(`Attempting to connect WebSocket to: ${wsUrl}`);
      connectWebSocket(wsUrl);

      // Send the initial instruction immediately after attempting WS connection
      // Note: This relies on the hook handling the actual sending once connected.
      // We might need to adjust based on the hook's connection timing guarantees.
      const initialWsRequest = { UserInput: { instruction: instruction } };
      // Queue the message; the hook will send it when isConnected becomes true.
      // Consider adding a check or callback in the hook if immediate sending is critical.
      console.log('Queueing initial instruction to send via WebSocket:', initialWsRequest);
      // We need to ensure sendWsMessage can queue or handle being called before fully connected.
      // Assuming the hook or underlying WebSocket handles buffering/sending on connect.
      // If not, this might need to be triggered by an effect watching `isConnected`.
      // For now, let's attempt to send directly after connectWebSocket call.
       sendWsMessage(JSON.stringify(initialWsRequest));

    } catch (error) {
      console.error('Failed to initialize session:', error);
      setMessages((prev) => [...prev, { type: 'error', text: `Failed to initialize session: ${error.message}` }]);
    }
  }, [sessionId, connectWebSocket, serverAddress]); // Added setIsProcessing dependency implicitly via scope

  const terminateSession = useCallback(async () => {
    if (!sessionId) return;
    console.log('Terminating session:', sessionId);
    try {
      const response = await fetch(`http://localhost:8080/api/v1/session/${sessionId}/terminate`, { method: 'POST' });
      if (!response.ok) {
        let errorBody = 'Unknown error';
        try { errorBody = await response.text(); } catch (_) {}
        throw new Error(`HTTP error! status: ${response.status}, message: ${errorBody}`);
      }
      console.log('Session terminated');
      disconnectWebSocket();
      setSessionId(null);
      setIsProcessing(false); // Stop processing on termination
      setMessages((prev) => [...prev, { type: 'system', text: 'Session terminated.' }]);
    } catch (error) {
      console.error('Failed to terminate session:', error);
      setMessages((prev) => [...prev, { type: 'error', text: `Failed to terminate session: ${error.message}` }]);
    }
  }, [sessionId, disconnectWebSocket]); // Added setIsProcessing dependency implicitly via scope

  const postUserMessage = useCallback((messageText) => {
    if (!messageText) return;
    if (!sessionId) {
      initializeSession(messageText);
    } else if (isConnected) {
      console.log('Sending user message:', messageText);
      const wsRequest = { UserInput: { instruction: messageText } };
      setIsProcessing(true); // Assume sending input starts processing
      sendWsMessage(JSON.stringify(wsRequest));
      setMessages((prevMessages) => [...prevMessages, { type: 'user', text: messageText }]);
    } else {
      console.warn('Cannot send message: Not connected or session not initialized.');
       setMessages((prev) => [...prev, { type: 'error', text: 'Cannot send message. Connection lost or session not active.' }]);
    }
  }, [sessionId, isConnected, sendWsMessage, initializeSession]); // Added setIsProcessing dependency implicitly via scope

  return (
    // Apply structure from style_grok.html
    <div className="flex flex-col min-h-screen bg-[#f9f8f6] text-gray-800">
      {/* Header */}
      <header className="flex items-center justify-between px-6 py-4 border-b border-gray-200">
        <div className="text-xl font-semibold">🔍 Loupe.ai</div>
        <div className="flex items-center gap-4 text-gray-500 text-lg">
          <button title="Settings">⚙️</button>
          {/* Placeholder for user profile */}
          <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center">?</div>
        </div>
      </header>

      {/* Main Content Area - Chat Interface */}
      <main className="flex-grow flex flex-col items-center justify-start pt-4 px-4 md:px-6">
        {/* Integrate ChatInterface, removing its outer container styling */}
        <ChatInterface
          messages={messages}
          onSendMessage={postUserMessage}
          // Pass necessary states for ChatInterface to determine input disabled state
          sessionId={sessionId}
          isConnected={isConnected}
          isProcessing={isProcessing}
          onTerminate={terminateSession}
        />
      </main>

      {/* Footer is handled by InputArea within ChatInterface now */}
    </div>
  );
}

export default App;