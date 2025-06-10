import React from 'react';

interface AnimatedSliceProps {
  children: React.ReactNode;
  index: number;
  isActive?: boolean;
}

const AnimatedSlice: React.FC<AnimatedSliceProps> = ({ children, index, isActive }) => {
  const animationDelay = `${index * 0.1}s`;
  
  // Create a unique class name for this slice
  const sliceClass = `animated-slice-${index}`;
  
  return (
    <>
      <style>
        {`
          @keyframes sliceIn {
            0% {
              opacity: 0;
              transform: scale(0.8) rotate(-10deg);
            }
            80% {
              opacity: 1;
              transform: scale(1.02) rotate(0deg);
            }
            100% {
              opacity: 1;
              transform: scale(1) rotate(0deg);
            }
          }
          
          .${sliceClass} {
            opacity: 0;
            transform: scale(0.8);
            transform-origin: center;
            animation: sliceIn 0.6s ease-out ${animationDelay} forwards;
            transition: all 0.15s ease-in-out;
          }
        `}
      </style>
      <g className={sliceClass}>
        {children}
      </g>
    </>
  );
};

export default AnimatedSlice; 