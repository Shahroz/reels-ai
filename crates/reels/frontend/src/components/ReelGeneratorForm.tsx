import React from 'react'
import './ReelGeneratorForm.css'

interface ReelGeneratorFormProps {
  prompt: string
  productUrl: string
  duration: number
  isGenerating: boolean
  onPromptChange: (value: string) => void
  onProductUrlChange: (value: string) => void
  onDurationChange: (value: number) => void
  onGenerate: () => void
  onCancel: () => void
}

function ReelGeneratorForm({
  prompt,
  productUrl,
  duration,
  isGenerating,
  onPromptChange,
  onProductUrlChange,
  onDurationChange,
  onGenerate,
  onCancel,
}: ReelGeneratorFormProps) {
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!isGenerating && prompt.trim()) {
      onGenerate()
    }
  }

  return (
    <form className="reel-generator-form" onSubmit={handleSubmit}>
      <div className="form-group">
        <label htmlFor="prompt">Reel Prompt</label>
        <textarea
          id="prompt"
          value={prompt}
          onChange={(e) => onPromptChange(e.target.value)}
          placeholder="Describe the reel you want to generate... e.g., 'Create a 15-second promotional reel about a new fitness app with energetic music and dynamic visuals'"
          disabled={isGenerating}
          rows={4}
        />
      </div>
      
      <div className="form-group">
        <label htmlFor="productUrl">Product/Service URL (Optional)</label>
        <input
          type="url"
          id="productUrl"
          value={productUrl}
          onChange={(e) => onProductUrlChange(e.target.value)}
          placeholder="https://example.com/product"
          disabled={isGenerating}
        />
      </div>
      
      <div className="form-group">
        <label htmlFor="duration">Duration (seconds)</label>
        <input
          type="number"
          id="duration"
          value={duration}
          onChange={(e) => onDurationChange(parseInt(e.target.value) || 15)}
          min={5}
          max={60}
          disabled={isGenerating}
        />
      </div>
      
      <div className="button-group">
        <button
          type="submit"
          className="btn-primary"
          disabled={isGenerating || !prompt.trim()}
        >
          {isGenerating ? (
            <>
              <span className="loading"></span> Generating...
            </>
          ) : (
            'Generate Reel'
          )}
        </button>
        <button
          type="button"
          className="btn-secondary"
          onClick={onCancel}
          disabled={!isGenerating}
        >
          Cancel
        </button>
      </div>
    </form>
  )
}

export default ReelGeneratorForm

