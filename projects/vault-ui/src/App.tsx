import Portfolio from './components/Portfolio'

function App() {
  const handleSettingsClick = () => {
    console.log('Settings clicked');
    // TODO: Implement settings dialog
  };

  const handleAddNetworkClick = () => {
    console.log('Add network clicked');
    // TODO: Implement add network functionality
  };

  return (
    <Portfolio 
      onSettingsClick={handleSettingsClick}
      onAddNetworkClick={handleAddNetworkClick}
    />
  )
}

export default App
