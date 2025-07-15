// Re-export modal components from Chakra UI
// If these don't exist in the current version, we'll provide fallbacks
export {
  useDisclosure
} from '@chakra-ui/react';

// Fallback modal components (you may need to install @chakra-ui/modal if available)
export const Modal = ({ children, isOpen, onClose, ...props }: any) => 
  isOpen ? <div {...props}>{children}</div> : null;

export const ModalOverlay = ({ children, ...props }: any) => 
  <div style={{ position: 'fixed', inset: 0, backgroundColor: 'rgba(0,0,0,0.5)' }} {...props}>{children}</div>;

export const ModalContent = ({ children, ...props }: any) => 
  <div style={{ margin: 'auto', marginTop: '10vh', maxWidth: '500px', backgroundColor: 'white', borderRadius: '8px' }} {...props}>{children}</div>;

export const ModalHeader = ({ children, ...props }: any) => 
  <div style={{ padding: '1rem', borderBottom: '1px solid #e2e8f0' }} {...props}>{children}</div>;

export const ModalFooter = ({ children, ...props }: any) => 
  <div style={{ padding: '1rem', borderTop: '1px solid #e2e8f0' }} {...props}>{children}</div>;

export const ModalBody = ({ children, ...props }: any) => 
  <div style={{ padding: '1rem' }} {...props}>{children}</div>;

export const ModalCloseButton = ({ onClick, ...props }: any) => 
  <button onClick={onClick} style={{ position: 'absolute', top: '1rem', right: '1rem' }} {...props}>×</button>; 