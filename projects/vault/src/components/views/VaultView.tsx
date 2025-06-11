import { Box } from '@chakra-ui/react';
import { Portfolio } from '../Portfolio';

interface VaultViewProps {
  onNavigate?: (action: 'send' | 'receive') => void;
}

export const VaultView = ({ onNavigate }: VaultViewProps) => {
  return (
    <Box height="100%" bg="gray.900">
      <Portfolio onNavigate={onNavigate} />
    </Box>
  );
}; 