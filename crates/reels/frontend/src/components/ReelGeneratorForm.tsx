import React from 'react'
import './ReelGeneratorForm.css'

interface ReelGeneratorFormProps {
  productUrl: string
  duration: number
  instruction: string
  isGenerating: boolean
  onProductUrlChange: (value: string) => void
  onDurationChange: (value: number) => void
  onInstructionChange: (value: string) => void
  onGenerate: () => void
  onCancel: () => void
}

function ReelGeneratorForm({
  productUrl,
  duration,
  instruction,
  isGenerating,
  onProductUrlChange,
  onDurationChange,
  onInstructionChange,
  onGenerate,
  onCancel,
}: ReelGeneratorFormProps) {
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (!isGenerating && instruction.trim()) {
      onGenerate()
    }
  }

  return (
    <form className="reel-generator-form" onSubmit={handleSubmit}>
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
      
      <div className="form-group">
        <label htmlFor="instruction">Instructions for the Reel</label>
        <textarea
          id="instruction"
          value={instruction}
          onChange={(e) => onInstructionChange(e.target.value)}
          placeholder="Describe what you want in the reel..."
          disabled={isGenerating}
          rows={8}
        />
      </div>
      
      <div className="button-group">
        <button
          type="submit"
          className="btn-primary"
          disabled={isGenerating || !instruction.trim()}
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

