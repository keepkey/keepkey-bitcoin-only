declare module 'coinselect' {
  interface Input {
    txId: string;
    vout: number;
    value: number;
    script?: Buffer;
  }

  interface Output {
    address?: string;
    value?: number;
    script?: Buffer;
  }

  interface Result {
    inputs: Input[];
    outputs: Output[];
    fee: number;
  }

  function coinselect(
    utxos: Input[],
    targets: Output[],
    feeRate: number
  ): Result;

  export = coinselect;
}

declare module 'coinselect/split' {
  interface Input {
    txId: string;
    vout: number;
    value: number;
    script?: Buffer;
  }

  interface Output {
    address?: string;
    value?: number;
    script?: Buffer;
  }

  interface Result {
    inputs: Input[];
    outputs: Output[];
    fee: number;
  }

  function split(
    utxos: Input[],
    targets: Output[],
    feeRate: number
  ): Result;

  export = split;
} 