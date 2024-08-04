export interface OptimizationResult {
  max_dps: number;
  best_rolls: Array<{
    roll_type: string;
    value: number;
  }>;
  best_modules: Array<{
    name: string;
    module_type: string;
    effects: Array<{
      effect_type: string;
      value: number;
    }>;
  }>;
}

export interface WeaponData {
  [key: string]: OptimizationResult;
}

export interface WeaponDataMap {
  [weaponName: string]: WeaponData;
}