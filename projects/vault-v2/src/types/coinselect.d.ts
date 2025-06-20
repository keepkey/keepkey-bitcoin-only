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

  function coinSelect(utxos: Input[], targets: Output[], feeRate: number): Result;
  export = coinSelect;
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

  function coinSelectSplit(utxos: Input[], targets: Output[], feeRate: number): Result;
  export = coinSelectSplit;
} 