import './StatusSection.css'

export interface StatusMessage {
  message: string
  type: 'info' | 'success' | 'error' | 'warning'
}

interface StatusSectionProps {
  messages: StatusMessage[]
}

function StatusSection({ messages }: StatusSectionProps) {
  if (messages.length === 0) {
    return null
  }

  return (
    <div className="status-section">
      <h3>Status</h3>
      <div className="status-container">
        {messages.map((msg, index) => (
          <div key={index} className={`status-item ${msg.type}`}>
            {msg.message}
          </div>
        ))}
      </div>
    </div>
  )
}

export default StatusSection

