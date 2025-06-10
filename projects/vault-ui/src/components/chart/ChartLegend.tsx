import React from 'react';
import { Box, HStack, Text, VStack } from '@chakra-ui/react';
import { DonutChartItem } from './DonutChart';

// Theme colors
const theme = {
  bg: '#000000',
  cardBg: '#111111',
  gold: '#FFD700',
  goldHover: '#FFE135',
  border: '#222222',
};

interface ChartLegendProps {
  data: DonutChartItem[];
  total: number;
  formatValue?: (value: number) => string;
  activeIndex?: number | null;
  onHoverItem?: (index: number | null) => void;
}

const ChartLegend: React.FC<ChartLegendProps> = ({
  data,
  total,
  formatValue = (value) => value.toString(),
  activeIndex,
  onHoverItem,
}) => {
  // Calculate percentage for each item
  const calculatePercentage = (value: number) => {
    if (total === 0) return 0;
    return (value / total) * 100;
  };

  return (
    <HStack 
      gap={4} 
      justify="center" 
      align="center"
      wrap="wrap"
      maxW="100%"
    >
      {data.slice(0, 4).map((item, index) => {
        const percentage = calculatePercentage(item.value);
        const isActive = activeIndex === index;
        
        return (
          <Box
            key={`${item.name}-${index}`}
            cursor="pointer"
            opacity={activeIndex !== null && activeIndex !== undefined && !isActive ? 0.6 : 1}
            transform={isActive ? 'scale(1.05)' : 'scale(1)'}
            transition="all 0.2s ease-in-out"
            onMouseEnter={() => onHoverItem && onHoverItem(index)}
            onMouseLeave={() => onHoverItem && onHoverItem(null)}
            _hover={{
              transform: 'scale(1.05)',
              opacity: 1,
            }}
          >
            <HStack gap={2} align="center">
              {/* Color indicator */}
              <Box
                w="8px"
                h="8px"
                borderRadius="full"
                bg={item.color}
                boxShadow={isActive ? `0 0 8px ${item.color}` : 'none'}
                transition="all 0.2s ease-in-out"
              />
              
              {/* Network info */}
              <VStack gap={0} align="flex-start">
                <Text
                  fontSize="xs"
                  color="white"
                  fontWeight={isActive ? "bold" : "medium"}
                  lineHeight="1"
                >
                  {item.name}
                </Text>
                <Text
                  fontSize="xs"
                  color="gray.400"
                  lineHeight="1"
                >
                  {percentage.toFixed(1)}%
                </Text>
              </VStack>
            </HStack>
          </Box>
        );
      })}
      
      {/* Show "others" if there are more than 4 items */}
      {data.length > 4 && (
        <Box>
          <HStack gap={2} align="center">
            <Box
              w="8px"
              h="8px"
              borderRadius="full"
              bg="gray.500"
            />
            <VStack gap={0} align="flex-start">
              <Text fontSize="xs" color="gray.400" lineHeight="1">
                +{data.length - 4} more
              </Text>
            </VStack>
          </HStack>
        </Box>
      )}
    </HStack>
  );
};

export default ChartLegend; 