import { Routes, Route } from 'react-router-dom'
import Home from './pages/Home'
import ReelGenerator from './pages/ReelGenerator'
import './App.css'

function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/generator" element={<ReelGenerator />} />
    </Routes>
  )
}

export default App

