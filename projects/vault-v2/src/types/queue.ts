export type DeviceResponse = 
  | {
      Xpub: {
        request_id: string;
        device_id: string;
        path: string;
        xpub: string;
        success: boolean;
        error?: string;
      };
    }
  | {
      Address: {
        request_id: string;
        device_id: string;
        path: string;
        address: string;
        success: boolean;
        error?: string;
      };
    };

export interface QueueStatus {
  device_id: string;
  queue_length: number;
  processing: boolean;
  last_response?: DeviceResponse;
} 