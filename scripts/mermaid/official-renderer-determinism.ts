import type { PageHandle } from "./official-renderer-types";

const FIXED_TIME_ISO = "2026-01-01T00:00:00.000Z";
const RANDOM_INITIAL_STATE = 0x12345678;
const RANDOM_MULTIPLIER = 1664525;
const RANDOM_INCREMENT = 1013904223;
const RANDOM_MODULUS = 0x100000000;

interface DeterminismConfig {
  fixedTimeIso: string;
  randomInitialState: number;
  randomMultiplier: number;
  randomIncrement: number;
  randomModulus: number;
}

const CONFIG: DeterminismConfig = {
  fixedTimeIso: FIXED_TIME_ISO,
  randomInitialState: RANDOM_INITIAL_STATE,
  randomMultiplier: RANDOM_MULTIPLIER,
  randomIncrement: RANDOM_INCREMENT,
  randomModulus: RANDOM_MODULUS,
};

export class OfficialRendererDeterminism {
  static install(page: PageHandle): Promise<void> {
    return page.evaluate((config: DeterminismConfig) => {
      const fixedTime = Date.parse(config.fixedTimeIso);
      let randomState = config.randomInitialState;

      Math.random = () => {
        randomState = (config.randomMultiplier * randomState + config.randomIncrement) >>> 0;
        return randomState / config.randomModulus;
      };

      Date.now = () => fixedTime;
    }, CONFIG);
  }
}
