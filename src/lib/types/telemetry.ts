export interface ProcessGroup {
  name: string;
  pid: number;
  cpu: number;
  memoryMb: number;
  count: number;
}

export interface GpuStats {
  name: string;
  usage: number;
  vramUsedMb: number;
  vramTotalMb: number;
}

export interface SystemStats {
  cpu: { usage: number; frequencyGhz: number; temperatureC: number | null };
  memory: { usedGb: number; totalGb: number; percent: number; swapUsedGb: number; swapTotalGb: number };
  disk: { usedGb: number; totalGb: number; readMbS: number; writeMbS: number };
  network: { downloadMbS: number; uploadMbS: number };
  gpu: GpuStats | null;
  processes: ProcessGroup[];
}
