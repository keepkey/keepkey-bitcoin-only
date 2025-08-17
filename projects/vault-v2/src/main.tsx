import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { WalletProvider } from "./contexts/WalletContext";
import { ChakraProvider } from "./providers/ChakraProvider";
import { DialogProvider } from "./contexts/DialogContext";
import { BlockingActionsProvider } from "./contexts/BlockingActionsContext";
import { SettingsProvider } from "./contexts/SettingsContext";
import "./i18n"; // Initialize i18n

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <ChakraProvider>
      <SettingsProvider>
        <DialogProvider>
          <BlockingActionsProvider>
            <WalletProvider>
              <App />
            </WalletProvider>
          </BlockingActionsProvider>
        </DialogProvider>
      </SettingsProvider>
    </ChakraProvider>
  </React.StrictMode>,
);
