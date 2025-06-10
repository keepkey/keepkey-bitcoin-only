import { Box, BoxProps } from '@chakra-ui/react';
import { keyframes } from '@emotion/react';

const pulse = keyframes`
  0% {
    background-color: rgba(113, 128, 150, 0.1);
  }
  50% {
    background-color: rgba(113, 128, 150, 0.2);
  }
  100% {
    background-color: rgba(113, 128, 150, 0.1);
  }
`;

interface SkeletonProps extends BoxProps {
  height?: string | number;
  width?: string | number;
}

export const Skeleton = ({ height = '20px', width = '100%', ...props }: SkeletonProps) => {
  return (
    <Box
      height={height}
      width={width}
      bg="gray.200"
      borderRadius="md"
      animation={`${pulse} 2s ease-in-out infinite`}
      {...props}
    />
  );
};

interface SkeletonCircleProps extends BoxProps {
  size?: string | number;
}

export const SkeletonCircle = ({ size = '40px', ...props }: SkeletonCircleProps) => {
  return (
    <Box
      width={size}
      height={size}
      bg="gray.200"
      borderRadius="full"
      animation={`${pulse} 2s ease-in-out infinite`}
      {...props}
    />
  );
}; 