import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ChakraProvider } from "./providers/ChakraProvider";
import { DialogProvider } from "./contexts/DialogContext";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ChakraProvider>
      <DialogProvider>
        <App />
      </DialogProvider>
    </ChakraProvider>
  </React.StrictMode>,
);
