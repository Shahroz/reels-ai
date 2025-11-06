import { Link } from 'react-router-dom'

import './Home.css'

function Home() {
  return (
    <div className="home">
      <div className="home-container">
        <h1>🚀 Reels AI</h1>
        <p>Generate AI-powered reels using AgentLoop</p>
        <div className="home-actions">
          <Link to="/generator" className="btn-primary">
            🎬 Try the Reels Generator
          </Link>
        </div>
      </div>
    </div>
  )
}

export default Home

