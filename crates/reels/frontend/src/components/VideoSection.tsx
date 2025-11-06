import './VideoSection.css'

export interface VideoInfo {
  reelId?: string
  duration?: number
  url?: string
}

interface VideoSectionProps {
  url: string
  info?: VideoInfo
}

function VideoSection({ url, info }: VideoSectionProps) {
  return (
    <div className="video-section">
      <div className="video-container">
        <video src={url} controls preload="auto">
          Your browser does not support the video tag.
        </video>
      </div>
      {info && (
        <div className="video-info">
          <h3>🎬 Generated Reel</h3>
          {info.reelId && <p><strong>Reel ID:</strong> {info.reelId}</p>}
          {info.duration && <p><strong>Duration:</strong> {info.duration} seconds</p>}
          {info.url && (
            <p>
              <strong>URL:</strong>{' '}
              <a href={info.url} target="_blank" rel="noopener noreferrer">
                {info.url}
              </a>
            </p>
          )}
          <p><em>Note: Video is streamable with HTTP range request support</em></p>
        </div>
      )}
    </div>
  )
}

export default VideoSection

