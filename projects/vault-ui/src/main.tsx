import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ChakraProvider } from "./providers/ChakraProvider";

// Add global dark mode styles
const globalStyles = `
  body {
    background-color: #000000 !important;
    color: #ffffff !important;
    margin: 0;
    padding: 0;
  }
  
  #root {
    min-height: 100vh;
    background-color: #000000;
  }
`;

// Inject global styles
const styleElement = document.createElement('style');
styleElement.textContent = globalStyles;
document.head.appendChild(styleElement);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ChakraProvider>
      <App />
    </ChakraProvider>
  </React.StrictMode>,
);
