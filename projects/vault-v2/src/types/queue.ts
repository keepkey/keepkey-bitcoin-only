export interface QueueStatus {
  device_id: string;
  queue_length: number;
  processing: boolean;
  last_response?: {
    request_id: string;
    device_id: string;
    path: string;
    xpub: string;
    success: boolean;
    error?: string;
  };
} 