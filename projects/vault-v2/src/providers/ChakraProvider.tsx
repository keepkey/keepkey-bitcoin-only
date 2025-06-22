import { ChakraProvider as ChakraUIProvider, createSystem, defaultConfig, defineConfig } from '@chakra-ui/react'
import { ReactNode } from 'react'

interface ChakraProviderProps {
  children: ReactNode
}

// Create a custom config that forces dark mode
const customConfig = defineConfig({
  ...defaultConfig,
  theme: {
    ...defaultConfig.theme,
    // Force dark mode semantics
    semanticTokens: {
      colors: {
        // Override any light mode colors if needed
      }
    }
  },
  // Disable CSS color scheme to prevent browser/OS interference
  cssVarsRoot: ':where(html)',
  // Force dark mode
  conditions: {
    light: '.light-mode-never-used',
    dark: ':where(html)'
  }
})

const customSystem = createSystem(customConfig)

export function ChakraProvider({ children }: ChakraProviderProps) {
  return (
    <ChakraUIProvider value={customSystem}>
      <style>{`
        /* Force dark mode styles */
        :root {
          color-scheme: dark !important;
        }
        html {
          background-color: #1a202c !important;
          color: #ffffff !important;
        }
        body {
          background-color: #1a202c !important;
          color: #ffffff !important;
        }
      `}</style>
      {children}
    </ChakraUIProvider>
  )
} 