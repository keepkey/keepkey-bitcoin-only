import { Flex, FlexProps } from '@chakra-ui/react'

import { KeepKeyUILogo } from './keepkey-ui'

export const Logo = (props: FlexProps & { logo?: React.ReactNode }) => {
  const { logo } = props
  return (
    <>
      <Flex
        width="160px"
        border="2px solid #c8a75c"
        borderRadius="lg"
        boxShadow="0 0 16px 0 rgba(200, 167, 92, 0.18)"
        className="kk-logo-float"
        {...props}
      >
        {logo || <KeepKeyUILogo />}
      </Flex>
      <style>{`
        @keyframes kkLogoFloat {
          0% { transform: scale(0.85); }
          50% { transform: scale(1.15); }
          100% { transform: scale(0.85); }
        }
        .kk-logo-float {
          animation: kkLogoFloat 4.5s ease-in-out infinite;
          will-change: transform;
          transition: box-shadow 0.3s;
        }
        .kk-logo-float:hover {
          box-shadow: 0 0 32px 4px rgba(200, 167, 92, 0.28);
          animation-play-state: paused;
        }
      `}</style>
    </>
  )
}
