import { useState } from 'react'

import LogSection from '../components/LogSection'
import VideoSection from '../components/VideoSection'
import StatusSection from '../components/StatusSection'
import { useReelGeneration } from '../hooks/useReelGeneration'
import ReelGeneratorForm from '../components/ReelGeneratorForm'

import './ReelGenerator.css'

function ReelGenerator() {
  const [prompt, setPrompt] = useState('Create a 15-second promotional reel about sustainable energy solutions')
  const [productUrl, setProductUrl] = useState('')
  const [duration, setDuration] = useState(15)

  const {
    isGenerating,
    statusMessages,
    logEntries,
    videoUrl,
    videoInfo,
    generateReel,
    cancelGeneration,
  } = useReelGeneration()

  const handleGenerate = () => {
    generateReel(prompt, productUrl || null, duration)
  }

  const handleCancel = () => {
    cancelGeneration()
  }

  return (
    <div className="reel-generator">
      <div className="container">
        <div className="header">
          <h1>🎬 Reels Generator</h1>
          <p>Generate AI-powered reels using AgentLoop</p>
        </div>

        <div className="content">
          <ReelGeneratorForm
            prompt={prompt}
            productUrl={productUrl}
            duration={duration}
            isGenerating={isGenerating}
            onPromptChange={setPrompt}
            onProductUrlChange={setProductUrl}
            onDurationChange={setDuration}
            onGenerate={handleGenerate}
            onCancel={handleCancel}
          />

          {statusMessages.length > 0 && (
            <StatusSection messages={statusMessages} />
          )}

          {videoUrl && (
            <VideoSection url={videoUrl} info={videoInfo} />
          )}

          {logEntries.length > 0 && (
            <LogSection entries={logEntries} />
          )}
        </div>
      </div>
    </div>
  )
}

export default ReelGenerator
