import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { ChakraProvider } from "./providers/ChakraProvider";
import { DialogProvider } from "./contexts/DialogContext";
import { BlockingActionsProvider } from "./contexts/BlockingActionsContext";
import { WalletProvider } from "./contexts/WalletContext";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ChakraProvider>
      <DialogProvider>
        <BlockingActionsProvider>
          <WalletProvider>
            <App />
          </WalletProvider>
        </BlockingActionsProvider>
      </DialogProvider>
    </ChakraProvider>
  </React.StrictMode>,
);
