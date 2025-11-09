import { useState, useCallback, useRef } from 'react'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080'

export interface LogEntry {
  message: string
  type: 'info' | 'success' | 'error' | 'warning'
  timestamp: Date
}

export interface StatusMessage {
  message: string
  type: 'info' | 'success' | 'error' | 'warning'
}

export interface VideoInfo {
  reelId?: string
  duration?: number
  url?: string
}

export function useReelGeneration() {
  const [isGenerating, setIsGenerating] = useState(false)
  const [statusMessages, setStatusMessages] = useState<StatusMessage[]>([])
  const [logEntries, setLogEntries] = useState<LogEntry[]>([])
  const [videoUrl, setVideoUrl] = useState<string>('')
  const [videoInfo, setVideoInfo] = useState<VideoInfo | undefined>()

  const currentSessionIdRef = useRef<string | null>(null)
  const websocketRef = useRef<WebSocket | null>(null)
  const pollIntervalRef = useRef<NodeJS.Timeout | null>(null)

  const addStatus = useCallback((message: string, type: StatusMessage['type'] = 'info') => {
    setStatusMessages((prev) => [...prev, { message, type }])
  }, [])

  const addLog = useCallback((message: string, type: LogEntry['type'] = 'info') => {
    setLogEntries((prev) => [...prev, { message, type, timestamp: new Date() }])
  }, [])

  const clearStatus = useCallback(() => {
    setStatusMessages([])
  }, [])

  const clearLog = useCallback(() => {
    setLogEntries([])
  }, [])

  const terminateExistingSession = useCallback(async () => {
    if (!currentSessionIdRef.current) {
      return
    }

    try {
      addLog(`Terminating existing session: ${currentSessionIdRef.current}`, 'info')

      if (websocketRef.current) {
        websocketRef.current.close()
        websocketRef.current = null
        addLog('WebSocket connection closed', 'info')
      }

      if (pollIntervalRef.current) {
        clearInterval(pollIntervalRef.current)
        pollIntervalRef.current = null
      }

      const response = await fetch(`${API_BASE_URL}/loupe/session/${currentSessionIdRef.current}/terminate`, {
        method: 'POST',
      })

      if (response.ok) {
        addLog(`Session ${currentSessionIdRef.current} terminated successfully`, 'success')
      } else {
        addLog(`Failed to terminate session: HTTP ${response.status}`, 'warning')
      }
    } catch (error) {
      addLog(`Error terminating session: ${error instanceof Error ? error.message : 'Unknown error'}`, 'warning')
    } finally {
      currentSessionIdRef.current = null
    }
  }, [addLog])

  const startResearch = useCallback(async (prompt: string, productUrl: string | null, duration: number) => {
    try {
      addLog('Starting research session...', 'info')
      addStatus('Creating research session...', 'info')

      let instruction = `Generate a ${duration}-second promotional reel. Prompt: "${prompt}"`

      if (productUrl) {
        instruction += ` Use information from this URL: ${productUrl}`
      }

      instruction += ` Use the generate_reel tool with prompt: "${prompt}" and time_range_seconds: ${duration}`

      if (productUrl) {
        instruction += ` and product_url: "${productUrl}"`
      }

      const response = await fetch(`${API_BASE_URL}/loupe/research`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          instruction: instruction,
          attachments: null,
        }),
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const data = await response.json()
      currentSessionIdRef.current = data.session_id

      addLog(`Research session started: ${currentSessionIdRef.current}`, 'success')
      addStatus(`Session created: ${currentSessionIdRef.current}`, 'success')

      return currentSessionIdRef.current
    } catch (error) {
      addLog(`Error starting research: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
      addStatus(`Error: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
      throw error
    }
  }, [addLog, addStatus])

  const handleReelGenerated = useCallback((toolResponse: any) => {
    try {
      const data = toolResponse.data

      if (data && data.reel_url) {
        let reelUrl = data.reel_url

        if (reelUrl.startsWith('/')) {
          reelUrl = `${API_BASE_URL}${reelUrl}`
        }

        const reelId = data.reel_id || 'unknown'
        const duration = data.duration_seconds

        addLog(`Reel generated successfully! URL: ${reelUrl}`, 'success')
        addStatus('Reel generated successfully!', 'success')

        setVideoUrl(reelUrl)
        setVideoInfo({
          reelId,
          duration,
          url: reelUrl,
        })
        setIsGenerating(false)
      } else {
        addLog('Reel generated but URL not found in response', 'warning')
        addStatus('Reel generated but URL missing', 'warning')
      }
    } catch (error) {
      addLog(`Error processing reel response: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
      addStatus(`Error processing response: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
    }
  }, [addLog, addStatus])

  const handleWebSocketEvent = useCallback((event: any) => {
    if ('ReasoningUpdate' in event) {
      addLog(`Agent reasoning: ${event.ReasoningUpdate}`, 'info')
      addStatus(`Agent: ${event.ReasoningUpdate.substring(0, 100)}...`, 'info')
    } else if ('ToolExecutionSuccess' in event) {
      const toolResponse = event.ToolExecutionSuccess
      addLog(`Tool executed: ${toolResponse.tool_name}`, 'success')

      if (toolResponse.tool_name === 'generate_reel') {
        handleReelGenerated(toolResponse)
      }
    } else if ('ToolExecutionFailure' in event) {
      const failure = event.ToolExecutionFailure
      addLog(`Tool failed: ${failure.error}`, 'error')
      addStatus(`Error: ${failure.error}`, 'error')
    } else if ('AgentAnswer' in event) {
      const answer = event.AgentAnswer
      addLog(`Agent answer: ${answer.content}`, 'info')
      if (answer.is_final) {
        addStatus('Final answer received', 'success')
      }
    } else if ('ResearchAnswer' in event) {
      const researchAnswer = event.ResearchAnswer
      addLog(`Research complete: ${researchAnswer.title}`, 'success')
      addStatus(`Research complete: ${researchAnswer.title}`, 'success')
    } else if ('StatusUpdate' in event) {
      const status = event.StatusUpdate
      addLog(`Status: ${status.status}`, 'info')
    } else if ('SessionTerminated' in event) {
      const termination = event.SessionTerminated
      addLog(`Session terminated: ${termination.reason || 'No reason provided'}`, 'warning')
      addStatus('Session terminated', 'warning')
      setIsGenerating(false)
    }
  }, [addLog, addStatus, handleReelGenerated])

  const connectWebSocket = useCallback((sessionId: string, handleEvent: (event: any) => void): Promise<void> => {
    return new Promise((resolve, reject) => {
      try {
        const wsUrl = API_BASE_URL.replace('http://', 'ws://').replace('https://', 'wss://')
        const wsPath = `${wsUrl}/loupe/session/${sessionId}/stream`

        addLog(`Connecting to WebSocket: ${wsPath}`, 'info')

        const ws = new WebSocket(wsPath)
        websocketRef.current = ws

        ws.onopen = () => {
          addLog('WebSocket connected', 'success')
          addStatus('Connected to real-time updates', 'success')
          resolve()
        }

        ws.onmessage = (event) => {
          try {
            const data = JSON.parse(event.data)
            handleEvent(data)
          } catch (error) {
            addLog(`Error parsing WebSocket message: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
          }
        }

        ws.onerror = (error) => {
          addLog(`WebSocket error: ${error}`, 'error')
          reject(error)
        }

        ws.onclose = () => {
          addLog('WebSocket disconnected', 'warning')
          if (isGenerating) {
            addStatus('Connection closed. You can check status manually.', 'warning')
          }
        }
      } catch (error) {
        reject(error)
      }
    })
  }, [addLog, addStatus, isGenerating])

  const checkStatus = useCallback(async (sessionId: string) => {
    try {
      const response = await fetch(`${API_BASE_URL}/loupe/session/${sessionId}/status`)
      if (response.ok) {
        const status = await response.json()
        addLog(`Status check: ${status.status}`, 'info')
        return status
      }
    } catch (error) {
      addLog(`Error checking status: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
    }
    return null
  }, [addLog])

  const generateReel = useCallback(async (prompt: string, productUrl: string | null, duration: number) => {
    await terminateExistingSession()

    clearStatus()
    clearLog()
    setVideoUrl('')
    setVideoInfo(undefined)
    setIsGenerating(true)

    try {
      const sessionId = await startResearch(prompt, productUrl, duration)
      if (sessionId) {
        try {
          await connectWebSocket(sessionId, handleWebSocketEvent)
        } catch (wsError) {
          addLog(`WebSocket connection failed: ${wsError instanceof Error ? wsError.message : 'Unknown error'}. Will poll status instead.`, 'warning')

          pollIntervalRef.current = setInterval(async () => {
            if (!isGenerating) {
              if (pollIntervalRef.current) {
                clearInterval(pollIntervalRef.current)
                pollIntervalRef.current = null
              }
              return
            }

            const status = await checkStatus(sessionId)
            if (status && status.status === 'completed') {
              if (pollIntervalRef.current) {
                clearInterval(pollIntervalRef.current)
                pollIntervalRef.current = null
              }
              setIsGenerating(false)
            }
          }, 2000)
        }
      }
    } catch (error) {
      addLog(`Generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
      addStatus(`Generation failed: ${error instanceof Error ? error.message : 'Unknown error'}`, 'error')
      setIsGenerating(false)
    }
  }, [terminateExistingSession, clearStatus, clearLog, startResearch, connectWebSocket, checkStatus, addLog, addStatus, isGenerating, handleWebSocketEvent])

  const cancelGeneration = useCallback(async () => {
    await terminateExistingSession()
    setIsGenerating(false)
    addStatus('Generation cancelled', 'warning')
    addLog('Generation cancelled by user', 'warning')
  }, [terminateExistingSession, addStatus, addLog])

  return {
    isGenerating,
    statusMessages,
    logEntries,
    videoUrl,
    videoInfo,
    generateReel,
    cancelGeneration,
  }
}