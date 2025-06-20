// Create custom modal components for Chakra UI v3 compatibility
import { Box, CloseButton, useDisclosure } from '@chakra-ui/react'

export const Modal = ({ children, isOpen, onClose, ...props }: any) => {
  if (!isOpen) return null
  return (
    <Box position="fixed" inset={0} zIndex={1000} {...props}>
      {children}
    </Box>
  )
}

export const ModalOverlay = ({ children, ...props }: any) => (
  <Box
    position="fixed"
    inset={0}
    bg="blackAlpha.600"
    onClick={props.onClick}
    {...props}
  >
    {children}
  </Box>
)

export const ModalContent = ({ children, ...props }: any) => (
  <Box
    position="relative"
    maxW="md"
    mx="auto"
    mt="10vh"
    bg="white"
    borderRadius="md"
    boxShadow="lg"
    {...props}
  >
    {children}
  </Box>
)

export const ModalHeader = ({ children, ...props }: any) => (
  <Box px={6} py={4} fontSize="lg" fontWeight="semibold" {...props}>
    {children}
  </Box>
)

export const ModalBody = ({ children, ...props }: any) => (
  <Box px={6} py={2} {...props}>
    {children}
  </Box>
)

export const ModalFooter = ({ children, ...props }: any) => (
  <Box px={6} py={4} display="flex" justifyContent="flex-end" gap={3} {...props}>
    {children}
  </Box>
)

export const ModalCloseButton = ({ onClick, ...props }: any) => (
  <CloseButton
    position="absolute"
    right={2}
    top={2}
    onClick={onClick}
    {...props}
  />
)

export { useDisclosure } 