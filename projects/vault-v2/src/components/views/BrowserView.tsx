import { useState, useEffect, useRef } from 'react';
import { 
  Box, 
  Input, 
  Button, 
  Flex, 
  Text, 
  Stack,
  IconButton,
  Spinner
} from '@chakra-ui/react';
import { FaArrowLeft, FaArrowRight, FaRedo, FaHome, FaSearch, FaExternalLinkAlt } from 'react-icons/fa';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

export const BrowserView = () => {
  // Use proxy server for keepkey.com to avoid CORS issues
  const [url, setUrl] = useState('http://localhost:8080');
  const [inputUrl, setInputUrl] = useState('keepkey.com');
  const [isLoading, setIsLoading] = useState(true);
  const [canGoBack, setCanGoBack] = useState(false);
  const [canGoForward, setCanGoForward] = useState(false);
  const [showControls, setShowControls] = useState(false);
  const [hoverTimer, setHoverTimer] = useState<NodeJS.Timeout | null>(null);
  const [apiStatus, setApiStatus] = useState<'checking' | 'connected' | 'error'>('checking');
  const [pendingNavigation, setPendingNavigation] = useState<string | null>(null);
  const [externalLinkMessage, setExternalLinkMessage] = useState<string | null>(null);
  const iframeRef = useRef<HTMLIFrameElement>(null);

  // Check API connectivity on mount
  useEffect(() => {
    checkApiConnection();
  }, []);

  const checkApiConnection = async () => {
    try {
      const response = await fetch('http://localhost:1646/spec/swagger.json');
      if (response.ok) {
        setApiStatus('connected');
        console.log('✅ API server is connected at localhost:1646');
      } else {
        setApiStatus('error');
        console.error('❌ API server returned error:', response.status);
      }
    } catch (error) {
      setApiStatus('error');
      console.error('❌ Failed to connect to API server:', error);
    }
  };

  const handleNavigate = async () => {
    if (!inputUrl) return;
    
    // Use proxy server for KeepKey domains
    let formattedUrl = inputUrl;
    
    // Check if it's a KeepKey domain
    if (inputUrl.includes('keepkey.com') || inputUrl === 'vault' || inputUrl === 'vault.keepkey.com') {
      // Always use proxy for KeepKey domains
      formattedUrl = 'http://localhost:8080';
    } else if (!inputUrl.startsWith('http://') && !inputUrl.startsWith('https://')) {
      // For non-KeepKey domains, add https://
      formattedUrl = `https://${inputUrl}`;
    }
    
    // Notify backend of URL change
    try {
      await invoke('browser_navigate', { url: formattedUrl });
    } catch (error) {
      console.error('Failed to notify backend of navigation:', error);
    }
    
    // Update the UI
    setUrl(formattedUrl);
    setIsLoading(true);
  };

  const handleHome = () => {
    // Use proxy for keepkey.com
    const homeUrl = 'http://localhost:8080';
    setInputUrl('keepkey.com');
    setUrl(homeUrl);
    setIsLoading(true);
  };

  const handleRefresh = () => {
    setIsLoading(true);
    // Force iframe reload by changing src
    const iframe = iframeRef.current;
    if (iframe) {
      iframe.src = iframe.src;
    }
  };

  const handleBack = () => {
    // Note: iframe history navigation is limited due to security restrictions
    window.history.back();
  };

  const handleForward = () => {
    // Note: iframe history navigation is limited due to security restrictions
    window.history.forward();
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleNavigate();
    }
  };

  const handleIframeLoad = () => {
    setIsLoading(false);
    // Try to get the actual URL from iframe (limited by CORS)
    const iframe = iframeRef.current;
    if (iframe) {
      try {
        // Try to inject a script to handle link clicks (may be blocked by CORS)
        const iframeDoc = iframe.contentDocument || iframe.contentWindow?.document;
        if (iframeDoc && iframe.contentWindow?.location.origin === window.location.origin) {
          // Only works for same-origin content
          const links = iframeDoc.getElementsByTagName('a');
          for (let i = 0; i < links.length; i++) {
            const link = links[i];
            if (link.target === '_blank' || link.target === '_new') {
              link.addEventListener('click', (e) => {
                e.preventDefault();
                handleExternalLink(link.href);
              });
            }
          }
        }
        setInputUrl(iframe.contentWindow?.location.href || url);
      } catch (e) {
        // Cross-origin restrictions prevent accessing iframe content
        console.log('Cross-origin iframe, cannot access content');
        setInputUrl(url);
      }
    }
  };

  const handleExternalLink = async (linkUrl: string) => {
    console.log('External link clicked:', linkUrl);
    
    // Check if it's a documentation or external link
    if (linkUrl.includes('docs') || linkUrl.includes('github') || !linkUrl.includes('keepkey.com')) {
      // Open in external browser
      try {
        await invoke('open_url', { url: linkUrl });
        // Show a temporary message
        setExternalLinkMessage(`Opening ${linkUrl} in external browser...`);
        setTimeout(() => setExternalLinkMessage(null), 3000);
      } catch (error) {
        console.error('Failed to open external link:', error);
        // Fallback: navigate in iframe
        setUrl(linkUrl);
        setInputUrl(linkUrl);
      }
    } else {
      // Navigate within the iframe
      setUrl(linkUrl);
      setInputUrl(linkUrl);
    }
  };

  const handleMouseEnter = () => {
    // Clear any existing timer
    if (hoverTimer) {
      clearTimeout(hoverTimer);
    }
    
    // Set a 3-second timer to show controls
    const timer = setTimeout(() => {
      setShowControls(true);
    }, 3000);
    
    setHoverTimer(timer);
  };

  const handleMouseLeave = () => {
    // Clear the timer if mouse leaves before 3 seconds
    if (hoverTimer) {
      clearTimeout(hoverTimer);
      setHoverTimer(null);
    }
    
    // Hide controls immediately when mouse leaves
    setShowControls(false);
  };

  // Listen for backend navigation commands
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupEventListeners = async () => {
      try {
        // Listen for backend navigation commands
        unlisten = await listen('browser:navigate', (event) => {
          const { url: newUrl } = event.payload as { url: string };
          console.log('Received backend navigation command:', newUrl);
          
          // Handle special case for support.keepkey.com
          if (newUrl.includes('support.keepkey.com')) {
            // Navigate directly to support URL
            setUrl(newUrl);
            setInputUrl('support.keepkey.com');
            setIsLoading(true);
            
            // Also update the iframe directly to ensure navigation
            setTimeout(() => {
              const iframe = iframeRef.current;
              if (iframe && iframe.src !== newUrl) {
                iframe.src = newUrl;
              }
            }, 50);
          } else {
            setUrl(newUrl);
            setInputUrl(newUrl);
            setIsLoading(true);
          }
        });
      } catch (error) {
        console.error('Failed to set up browser event listeners:', error);
      }
    };

    setupEventListeners();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);
  
  // Handle pending navigation once component is ready
  useEffect(() => {
    if (pendingNavigation) {
      console.log('Processing pending navigation to:', pendingNavigation);
      setUrl(pendingNavigation);
      setInputUrl(pendingNavigation.includes('support.keepkey.com') ? 'support.keepkey.com' : pendingNavigation);
      setIsLoading(true);
      setPendingNavigation(null);
    }
  }, [pendingNavigation]);

      return (
      <Box height="100%" bg="rgba(0, 0, 0, 0.4)" display="flex" flexDirection="column">
      {/* Browser Toolbar - Hidden by default, shown after 3 second hover */}
      {showControls && (
        <Box 
          bg="gray.800" 
          borderBottom="1px solid" 
          borderColor="gray.700" 
          p={3}
          transition="all 0.3s ease"
          onMouseEnter={handleMouseEnter}
          onMouseLeave={handleMouseLeave}
        >
          <Stack direction="column" gap={3}>
            {/* Navigation Controls */}
            <Flex gap={2} align="center">
              <IconButton
                aria-label="Back"
                size="sm"
                variant="outline"
                colorScheme="gray"
                onClick={handleBack}
                disabled={!canGoBack}
              >
                <FaArrowLeft />
              </IconButton>
              <IconButton
                aria-label="Forward"
                size="sm"
                variant="outline"
                colorScheme="gray"
                onClick={handleForward}
                disabled={!canGoForward}
              >
                <FaArrowRight />
              </IconButton>
              <IconButton
                aria-label="Refresh"
                size="sm"
                variant="outline"
                colorScheme="gray"
                onClick={handleRefresh}
              >
                <FaRedo />
              </IconButton>
              <IconButton
                aria-label="Home"
                size="sm"
                variant="outline"
                colorScheme="blue"
                onClick={handleHome}
              >
                <FaHome />
              </IconButton>
              <IconButton
                aria-label="Open in External Browser"
                size="sm"
                variant="outline"
                colorScheme="green"
                onClick={() => handleExternalLink(url)}
                title="Open current page in external browser"
              >
                <FaExternalLinkAlt />
              </IconButton>
            </Flex>

            {/* Address Bar */}
            <Flex gap={2} align="center">
              <Input
                placeholder="Enter URL..."
                value={inputUrl}
                onChange={(e) => setInputUrl(e.target.value)}
                onKeyPress={handleKeyPress}
                bg="gray.700"
                border="1px solid"
                borderColor="gray.600"
                color="white"
                _placeholder={{ color: 'gray.400' }}
                _focus={{
                  borderColor: 'blue.500',
                  boxShadow: '0 0 0 1px var(--chakra-colors-blue-500)',
                }}
              />
              <Button
                colorScheme="blue"
                size="md"
                onClick={handleNavigate}
                disabled={isLoading}
              >
                <FaSearch />
              </Button>
            </Flex>
          </Stack>
        </Box>
      )}

      {/* Invisible hover area at top for triggering controls */}
      <Box
        position="absolute"
        top="0"
        left="0"
        right="0"
        height="20px"
        zIndex={5}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
      />

      {/* Browser Content */}
      <Box flex="1" position="relative" overflow="hidden">
        {isLoading && (
          <Flex
            position="absolute"
            top="20px"
            left="50%"
            transform="translateX(-50%)"
            align="center"
            gap={2}
            bg="gray.800"
            px={4}
            py={2}
            borderRadius="md"
            zIndex={10}
          >
            <Spinner size="sm" color="blue.400" />
            <Text color="white" fontSize="sm">Loading...</Text>
          </Flex>
        )}
        
        {externalLinkMessage && (
          <Flex
            position="absolute"
            bottom="20px"
            left="50%"
            transform="translateX(-50%)"
            align="center"
            gap={2}
            bg="green.700"
            px={4}
            py={2}
            borderRadius="md"
            zIndex={10}
          >
            <FaExternalLinkAlt color="white" />
            <Text color="white" fontSize="sm">{externalLinkMessage}</Text>
          </Flex>
        )}
        
        {/* Embedded Website */}
        <iframe
          ref={iframeRef}
          id="browser-iframe"
          src={url}
          style={{
            width: '100%',
            height: '100%',
            border: 'none',
          }}
          onLoad={handleIframeLoad}
          sandbox="allow-same-origin allow-scripts allow-popups allow-popups-to-escape-sandbox allow-forms allow-top-navigation allow-modals allow-downloads"
          allow="accelerometer; camera; encrypted-media; geolocation; gyroscope; microphone; midi; payment; usb"
          title="KeepKey Browser"
        />
      </Box>
    </Box>
  );
}; 