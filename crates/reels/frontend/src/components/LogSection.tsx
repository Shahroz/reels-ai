import { useEffect, useRef } from 'react'
import './LogSection.css'

export interface LogEntry {
  message: string
  type: 'info' | 'success' | 'error' | 'warning'
  timestamp: Date
}

interface LogSectionProps {
  entries: LogEntry[]
}

function LogSection({ entries }: LogSectionProps) {
  const logContainerRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight
    }
  }, [entries])

  if (entries.length === 0) {
    return null
  }

  return (
    <div className="log-section">
      <h3>Event Log</h3>
      <div className="log-container" ref={logContainerRef}>
        {entries.map((entry, index) => (
          <div key={index} className={`log-entry ${entry.type}`}>
            <span className="log-timestamp">
              [{entry.timestamp.toLocaleTimeString()}]
            </span>{' '}
            {entry.message}
          </div>
        ))}
      </div>
    </div>
  )
}

export default LogSection

