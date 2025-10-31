
import { useState, useEffect, useRef, useCallback } from 'react';

const useAgentLoopWebSocket = (serverAddress, onMessageCallback) => {
  const [ws, setWs] = useState(null);
  const [isConnected, setIsConnected] = useState(false);
  const reconnectAttemptRef = useRef(0);
  // Removed heartbeat refs and constants

  // Removed clearHeartbeatTimers function
  // Removed setupHeartbeat function

  const connectWebSocket = useCallback((url) => {
    if (!url) {
      console.error("Cannot connect WebSocket without URL.");
      return;
    }
    if (ws && ws.readyState === WebSocket.OPEN && ws.url === url) {
      console.log("WebSocket already connected to this URL.");
      return;
    }
    if (ws) {
        console.log("Closing existing WebSocket connection before reconnecting.");
        ws.close();
    }

    console.log(`Attempting to connect WebSocket to: ${url}`);
    const webSocket = new WebSocket(url);

    webSocket.onopen = () => {
      console.log(`WebSocket connected to: ${url}`);
      setIsConnected(true);
      reconnectAttemptRef.current = 0;
      setWs(webSocket);
      // Removed setupHeartbeat(webSocket) call;
    };

    webSocket.onmessage = (event) => {
      // console.log('WebSocket message received:', event.data);

      // Removed pong handling logic

      try {
        const parsedData = JSON.parse(event.data);
        if (onMessageCallback) {
          onMessageCallback(parsedData);
        }
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error);
        if (onMessageCallback) {
          onMessageCallback({ type: 'raw', data: event.data });
        }
      }
    };

    webSocket.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    webSocket.onclose = (event) => {
      const closedUrl = ws ? ws.url : 'unknown URL';
      console.log(`WebSocket disconnected from: ${closedUrl}. Code: ${event.code}, Reason: ${event.reason}`);
      setIsConnected(false);
      setWs(null);
      // Removed clearHeartbeatTimers() call;

      // Optional: Reconnect logic (remains commented out)
    };

    setWs(webSocket);

  }, [serverAddress, onMessageCallback, ws]); // Removed dependencies related to heartbeat

  const disconnectWebSocket = useCallback(() => {
    if (ws) {
      console.log('Disconnecting WebSocket...');
      ws.close();
      setWs(null);
      setIsConnected(false);
      // Removed clearHeartbeatTimers() call;
    }
  }, [ws]); // Removed dependencies related to heartbeat

  const sendMessage = useCallback((message) => {
    if (ws && ws.readyState === WebSocket.OPEN) {
      ws.send(message);
    } else {
      console.error('WebSocket is not connected. Cannot send message.');
    }
  }, [ws]);

  useEffect(() => {
    return () => {
      disconnectWebSocket();
    };
  }, [disconnectWebSocket]);

  return { ws, isConnected, connectWebSocket, disconnectWebSocket, sendMessage };
};

export default useAgentLoopWebSocket;
